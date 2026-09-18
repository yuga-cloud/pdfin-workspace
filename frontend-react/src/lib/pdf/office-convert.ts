import {
  blobFromBytes,
  stem,
  yieldToUi,
} from "@/lib/pdf/bytes";
import type {
  ProgressFn,
  ProcessOutput,
} from "./engine";
import {
  loadPdfjs,
  MAX_DEVICE_OUTPUT_BYTES,
  MAX_DEVICE_PDF_BYTES,
  MAX_DEVICE_PDF_PAGES,
} from "./engine";

async function loadPdfLib() {
  return import("pdf-lib");
}

async function loadMammoth() {
  return import("mammoth");
}

async function loadJsZip() {
  return import("jszip");
}

async function loadDocx() {
  return import("docx");
}

async function loadPptxGenJs() {
  return import("pptxgenjs");
}

const MAX_OFFICE_ARCHIVE_BYTES = 100 * 1024 * 1024;
const MAX_OFFICE_ARCHIVE_ENTRIES = 2_048;
const MAX_OFFICE_ARCHIVE_UNCOMPRESSED_BYTES = 256 * 1024 * 1024;
const MAX_OFFICE_ENTRY_BYTES = 32 * 1024 * 1024;
const MAX_OFFICE_RELATIONSHIP_BYTES = 1024 * 1024;
const MAX_OFFICE_RELATIONSHIP_SCAN_BYTES = 8 * 1024 * 1024;
const MAX_PPT_SLIDES = 500;
const MAX_DEVICE_TEXT_BYTES = 8 * 1024 * 1024;
const MAX_PDF_TEXT_ITEMS_PER_PAGE = 10_000;
const MAX_PDF_TEXT_ITEMS_TOTAL = 250_000;

type JsZipEntryMeta = {
  uncompressedSize?: unknown;
};

function officeEntrySize(entry: unknown, label: string): number {
  const meta = (entry as { _data?: JsZipEntryMeta })._data;
  const size = meta?.uncompressedSize;

  if (
    typeof size !== "number" ||
    !Number.isSafeInteger(size) ||
    size < 0
  ) {
    throw new Error(
      "Ukuran entry " + label + " tidak dapat divalidasi dengan aman.",
    );
  }

  return size;
}

function validateOfficeEntryName(
  entry: { name?: unknown; unsafeOriginalName?: unknown },
  label: string,
): string {
  const raw = String(
    entry.unsafeOriginalName ?? entry.name ?? "",
  );
  const normalized = raw.replaceAll("\\", "/");

  if (
    !normalized ||
    normalized.startsWith("/") ||
    normalized.split("/").some((part) => part === "..")
  ) {
    throw new Error(
      "File " + label + " memiliki entry ZIP dengan path berbahaya.",
    );
  }

  return normalized;
}

async function validateOfficeRelationships(
  zip: {
    files: Record<string, {
      dir: boolean;
      name: string;
      unsafeOriginalName?: string;
      async: (type: "string") => Promise<string>;
      [key: string]: unknown;
    }>;
  },
  label: string,
): Promise<void> {
  let scannedBytes = 0;

  for (const entry of Object.values(zip.files)) {
    const name = validateOfficeEntryName(entry, label);

    if (entry.dir || !name.toLowerCase().endsWith(".rels")) {
      continue;
    }

    const size = officeEntrySize(entry, label);

    if (size > MAX_OFFICE_RELATIONSHIP_BYTES) {
      throw new Error(
        "Relationship XML " + label + " terlalu besar untuk diproses dengan aman.",
      );
    }

    scannedBytes += size;
    if (scannedBytes > MAX_OFFICE_RELATIONSHIP_SCAN_BYTES) {
      throw new Error(
        "Total Relationship XML " + label + " melebihi batas validasi.",
      );
    }

    const xml = await entry.async("string");
    const normalized = xml
      .replace(/\s+/g, "")
      .toLowerCase();

    if (
      normalized.includes('targetmode="external"') ||
      normalized.includes("targetmode='external'")
    ) {
      throw new Error(
        "File " + label + " mengandung external relationship yang tidak didukung.",
      );
    }
  }
}

async function loadSafeOfficeZip(
  data: ArrayBuffer,
  label: string,
  requiredEntry: string,
) {
  if (data.byteLength > MAX_OFFICE_ARCHIVE_BYTES) {
    throw new Error(
      "File " + label + " terlalu besar untuk diproses di perangkat (maksimum 100 MiB).",
    );
  }

  const { default: JSZip } = await loadJsZip();

  let zip;
  try {
    zip = await JSZip.loadAsync(data);
  } catch {
    throw new Error("File " + label + " bukan arsip Office yang valid.");
  }

  const entries = Object.values(zip.files);
  if (
    entries.length === 0 ||
    entries.length > MAX_OFFICE_ARCHIVE_ENTRIES
  ) {
    throw new Error(
      "File " + label + " memiliki jumlah entry ZIP yang tidak didukung.",
    );
  }

  if (!zip.file(requiredEntry)) {
    throw new Error(
      "File " + label + " tidak memiliki entry Office wajib.",
    );
  }

  let totalUncompressed = 0;

  for (const entry of entries) {
    const name = validateOfficeEntryName(entry, label);
    if (entry.dir) continue;

    const size = officeEntrySize(entry, label);

    if (size > MAX_OFFICE_ENTRY_BYTES) {
      throw new Error(
        "Entry ZIP " + label + " terlalu besar untuk diproses di perangkat.",
      );
    }

    totalUncompressed += size;
    if (totalUncompressed > MAX_OFFICE_ARCHIVE_UNCOMPRESSED_BYTES) {
      throw new Error(
        "Ukuran terurai file " + label + " melebihi batas maksimum.",
      );
    }

    const normalizedName = name.toLowerCase();
    if (
      normalizedName.includes("vbaproject.bin") ||
      normalizedName.includes("/externallinks/")
    ) {
      throw new Error(
        "File " + label + " mengandung macro/external link yang tidak didukung.",
      );
    }
  }

  await validateOfficeRelationships(zip, label);
  return zip;
}

type PdfTextItem = {
  transform: number[];
  str: string;
};

export function isPdfTextItem(item: unknown): item is PdfTextItem {
  if (!item || typeof item !== "object") {
    return false;
  }

  const candidate = item as {
    transform?: unknown;
    str?: unknown;
  };

  return (
    Array.isArray(candidate.transform) &&
    candidate.transform.length >= 6 &&
    candidate.transform.every(
      (value) =>
        typeof value === "number" && Number.isFinite(value),
    ) &&
    typeof candidate.str === "string"
  );
}

function getPdfTextItems(items: unknown[]): PdfTextItem[] {
  return items.filter(isPdfTextItem);
}

// ─────────────────────────────────────────────────────────────
// Word → PDF
// ─────────────────────────────────────────────────────────────

export async function wordToPdf(
  file: File,
  onProgress?: ProgressFn,
): Promise<ProcessOutput> {
  onProgress?.(
    0.1,
    1,
    "Membaca file Word...",
  );

  const arrayBuffer =
    await file.arrayBuffer();

  await loadSafeOfficeZip(
    arrayBuffer,
    "Word",
    "word/document.xml",
  );

  const { default: mammoth } =
    await loadMammoth();

  onProgress?.(
    0.3,
    1,
    "Mengekstrak teks...",
  );

  const result =
    await mammoth.extractRawText({
      arrayBuffer,
    });

  const text =
    result.value;

  if (text.length > MAX_DEVICE_TEXT_BYTES) {
    throw new Error(
      "Teks hasil ekstraksi Word terlalu besar untuk diproses di perangkat.",
    );
  }

  const {
    PDFDocument,
    StandardFonts,
    rgb,
  } = await loadPdfLib();

  onProgress?.(
    0.6,
    1,
    "Menghasilkan PDF...",
  );

  const pdfDoc =
    await PDFDocument.create();

  const font =
    await pdfDoc.embedFont(
      StandardFonts.Helvetica,
    );

  const fontSize = 11;
  const leading = 14;
  const margin = 50;

  const pageWidth = 595.28;
  const pageHeight = 841.89;

  const maxLineWidth =
    pageWidth - margin * 2;

  let page =
    pdfDoc.addPage([
      pageWidth,
      pageHeight,
    ]);

  let y =
    pageHeight - margin;

  const paragraphs =
    text.split("\n");

  for (
    const paragraph of paragraphs
  ) {
    const trimmed =
      paragraph.trim();

    if (!trimmed) {
      y -= leading;
      continue;
    }

    const words =
      trimmed.split(/\s+/);

    let currentLine = "";

    for (
      const word of words
    ) {
      const testLine =
        currentLine
          ? `${currentLine} ${word}`
          : word;

      const width =
        font.widthOfTextAtSize(
          testLine,
          fontSize,
        );

      if (
        width >
        maxLineWidth
      ) {
        if (!currentLine) {
          page =
            pdfDoc.addPage([
              pageWidth,
              pageHeight,
            ]);

          y =
            pageHeight -
            margin;

          currentLine =
            word;

          continue;
        }

        if (
          y <
          margin + leading
        ) {
          page =
            pdfDoc.addPage([
              pageWidth,
              pageHeight,
            ]);

          y =
            pageHeight -
            margin;
        }

        page.drawText(
          currentLine,
          {
            x: margin,
            y,
            size: fontSize,
            font,
            color: rgb(
              0.1,
              0.1,
              0.1,
            ),
          },
        );

        y -= leading;

        currentLine =
          word;
      } else {
        currentLine =
          testLine;
      }
    }

    if (currentLine) {
      if (
        y <
        margin + leading
      ) {
        page =
          pdfDoc.addPage([
            pageWidth,
            pageHeight,
          ]);

        y =
          pageHeight -
          margin;
      }

      page.drawText(
        currentLine,
        {
          x: margin,
          y,
          size: fontSize,
          font,
          color: rgb(
            0.1,
            0.1,
            0.1,
          ),
        },
      );

      y -= leading;
    }

    y -= leading / 2;

    await yieldToUi();
  }

  onProgress?.(
    0.9,
    1,
    "Menyimpan...",
  );

  const pdfBytes =
    await pdfDoc.save();

  if (pdfBytes.byteLength > MAX_DEVICE_OUTPUT_BYTES) {
    throw new Error(
      "PDF hasil Word terlalu besar untuk diproses di perangkat.",
    );
  }

  onProgress?.(
    1,
    1,
    "Selesai!",
  );

  return {
    blob: blobFromBytes(
      pdfBytes,
      "application/pdf",
    ),
    filename:
      `${stem(file.name)}.pdf`,
    mime:
      "application/pdf",
  };
}

// ─────────────────────────────────────────────────────────────
// PowerPoint → PDF
// ─────────────────────────────────────────────────────────────

export async function powerpointToPdf(
  file: File,
  onProgress?: ProgressFn,
): Promise<ProcessOutput> {
  onProgress?.(
    0.1,
    1,
    "Membaca file PowerPoint...",
  );

  const arrayBuffer =
    await file.arrayBuffer();

  const zip =
    await loadSafeOfficeZip(
      arrayBuffer,
      "PowerPoint",
      "ppt/presentation.xml",
    );

  const slideFiles =
    Object.keys(zip.files)
      .filter(
        (name) =>
          name.startsWith(
            "ppt/slides/slide",
          ) &&
          name.endsWith(".xml"),
      );

  slideFiles.sort(
    (a, b) => {
      const numA =
        parseInt(
          a.replace(
            /[^\d]/g,
            "",
          ),
          10,
        );

      const numB =
        parseInt(
          b.replace(
            /[^\d]/g,
            "",
          ),
          10,
        );

      return numA - numB;
    },
  );

  if (
    slideFiles.length === 0
  ) {
    throw new Error(
      "File PowerPoint tidak memiliki slide.",
    );
  }

  if (slideFiles.length > MAX_PPT_SLIDES) {
    throw new Error(
      "PowerPoint memiliki terlalu banyak slide untuk diproses di perangkat.",
    );
  }

  const {
    PDFDocument,
    StandardFonts,
    rgb,
  } = await loadPdfLib();

  onProgress?.(
    0.6,
    1,
    "Menghasilkan PDF...",
  );

  const pdfDoc =
    await PDFDocument.create();

  const font =
    await pdfDoc.embedFont(
      StandardFonts.Helvetica,
    );

  const fontBold =
    await pdfDoc.embedFont(
      StandardFonts.HelveticaBold,
    );

  const pageWidth = 841.89;
  const pageHeight = 595.28;
  const margin = 50;

  const parser =
    new DOMParser();

  for (
    let index = 0;
    index < slideFiles.length;
    index += 1
  ) {
    onProgress?.(
      0.6 +
        (index /
          slideFiles.length) *
          0.3,
      1,
      `Memproses slide ${index + 1}/${slideFiles.length}...`,
    );

    const slideEntry =
      zip.files[slideFiles[index]];
    if (!slideEntry) {
      throw new Error("Slide PowerPoint tidak ditemukan di dalam arsip.");
    }

    const slideSize = officeEntrySize(
      slideEntry,
      "PowerPoint",
    );

    if (slideSize > MAX_OFFICE_ENTRY_BYTES) {
      throw new Error(
        "Ukuran slide PowerPoint terlalu besar untuk diproses di perangkat.",
      );
    }

    const slideXmlText =
      await slideEntry.async("text");

    if (slideXmlText.length > MAX_DEVICE_TEXT_BYTES) {
      throw new Error(
        "XML slide PowerPoint terlalu besar untuk diproses di perangkat.",
      );
    }

    const xmlDoc =
      parser.parseFromString(
        slideXmlText,
        "text/xml",
      );

    const textNodes =
      xmlDoc.getElementsByTagName(
        "a:t",
      );

    const textRuns: string[] = [];

    for (
      let j = 0;
      j < textNodes.length;
      j += 1
    ) {
      const nodeText =
        textNodes[j].textContent;

      if (
        nodeText &&
        nodeText.trim()
      ) {
        textRuns.push(
          nodeText.trim(),
        );
      }
    }

    const page =
      pdfDoc.addPage([
        pageWidth,
        pageHeight,
      ]);

    page.drawRectangle({
      x: 10,
      y: 10,
      width:
        pageWidth - 20,
      height:
        pageHeight - 20,
      borderWidth: 1,
      borderColor: rgb(
        0.8,
        0.8,
        0.8,
      ),
    });

    let y =
      pageHeight - margin;

    if (
      textRuns.length > 0
    ) {
      const title =
        textRuns[0];

      page.drawText(
        title,
        {
          x: margin,
          y,
          size: 24,
          font: fontBold,
          color: rgb(
            0.1,
            0.2,
            0.4,
          ),
        },
      );

      y -= 40;

      for (
        let j = 1;
        j < textRuns.length;
        j += 1
      ) {
        const text =
          textRuns[j];

        if (
          y <
          margin + 20
        ) {
          break;
        }

        page.drawText(
          `• ${text}`,
          {
            x:
              margin + 20,
            y,
            size: 14,
            font,
            color: rgb(
              0.2,
              0.2,
              0.2,
            ),
          },
        );

        y -= 25;
      }
    } else {
      page.drawText(
        "[Slide Kosong]",
        {
          x: margin,
          y:
            pageHeight / 2,
          size: 14,
          font,
          color: rgb(
            0.5,
            0.5,
            0.5,
          ),
        },
      );
    }

    await yieldToUi();
  }

  onProgress?.(
    0.9,
    1,
    "Menyimpan...",
  );

  const pdfBytes =
    await pdfDoc.save();

  onProgress?.(
    1,
    1,
    "Selesai!",
  );

  return {
    blob: blobFromBytes(
      pdfBytes,
      "application/pdf",
    ),
    filename:
      `${stem(file.name)}.pdf`,
    mime:
      "application/pdf",
  };
}

// ─────────────────────────────────────────────────────────────
// PDF → Word
// ─────────────────────────────────────────────────────────────

export async function pdfToWord(
  file: File,
  onProgress?: ProgressFn,
): Promise<ProcessOutput> {
  onProgress?.(
    0.1,
    1,
    "Memuat engine PDF...",
  );

  const pdfjs =
    await loadPdfjs();

  const {
    Document,
    Packer,
    Paragraph,
    TextRun,
  } = await loadDocx();

  const data =
    new Uint8Array(
      await file.arrayBuffer(),
    );

  if (data.byteLength > MAX_DEVICE_PDF_BYTES) {
    throw new Error(
      "PDF terlalu besar untuk diproses di perangkat (maksimum 100 MiB).",
    );
  }

  const task =
    pdfjs.getDocument({
      data,
      stopAtErrors: true,
      maxImageSize: 20_000_000,
    });

  const pdf =
    await task.promise;

  if (pdf.numPages === 0) {
    throw new Error("PDF tidak memiliki halaman.");
  }

  if (pdf.numPages > MAX_DEVICE_PDF_PAGES) {
    throw new Error(
      "PDF memiliki terlalu banyak halaman untuk diproses di perangkat (maksimum " +
        MAX_DEVICE_PDF_PAGES +
        ").",
    );
  }

  try {
    const total =
      pdf.numPages;

    const docParagraphs:
      InstanceType<
        typeof Paragraph
      >[] = [];

    let totalTextItems = 0;

    for (
      let pageNumber = 1;
      pageNumber <= total;
      pageNumber += 1
    ) {
      onProgress?.(
        0.1 +
          (pageNumber /
            total) *
            0.7,
        1,
        `Mengekstrak teks halaman ${pageNumber}...`,
      );

      const page =
        await pdf.getPage(
          pageNumber,
        );

      try {
        const textContent =
          await page.getTextContent();

        const items =
          getPdfTextItems(textContent.items);

        if (items.length > MAX_PDF_TEXT_ITEMS_PER_PAGE) {
          throw new Error(
            "Terlalu banyak item teks pada halaman PDF.",
          );
        }

        totalTextItems += items.length;
        if (totalTextItems > MAX_PDF_TEXT_ITEMS_TOTAL) {
          throw new Error(
            "Jumlah total item teks PDF terlalu besar untuk diproses di perangkat.",
          );
        }

        const sortedItems =
          [...items].sort(
            (a, b) => {
              const yA =
                a.transform[5];

              const yB =
                b.transform[5];

              if (
                Math.abs(
                  yA - yB,
                ) < 5
              ) {
                return (
                  a.transform[4] -
                  b.transform[4]
                );
              }

              return yB - yA;
            },
          );

        let currentY = -1;
        let currentLine = "";

        docParagraphs.push(
          new Paragraph({
            children: [
              new TextRun({
                text:
                  `--- Halaman ${pageNumber} ---`,
                bold: true,
              }),
            ],
          }),
        );

        for (
          const item of sortedItems
        ) {
          const y =
            item.transform[5];

          if (
            currentY === -1
          ) {
            currentY = y;
            currentLine =
              item.str;
          } else if (
            Math.abs(
              currentY - y,
            ) < 5
          ) {
            currentLine +=
              currentLine.endsWith(
                " ",
              ) ||
              item.str.startsWith(
                " ",
              )
                ? ""
                : " ";

            currentLine +=
              item.str;
          } else {
            if (
              currentLine.trim()
            ) {
              docParagraphs.push(
                new Paragraph({
                  children: [
                    new TextRun(
                      currentLine.trim(),
                    ),
                  ],
                }),
              );
            }

            currentY = y;
            currentLine =
              item.str;
          }
        }

        if (
          currentLine.trim()
        ) {
          docParagraphs.push(
            new Paragraph({
              children: [
                new TextRun(
                  currentLine.trim(),
                ),
              ],
            }),
          );
        }

        docParagraphs.push(
          new Paragraph({
            children: [],
          }),
        );
      } finally {
        page.cleanup();
      }

      await yieldToUi();
    }

    onProgress?.(
      0.85,
      1,
      "Mengemas dokumen Word...",
    );

    const doc =
      new Document({
        sections: [
          {
            properties: {},
            children:
              docParagraphs,
          },
        ],
      });

    const docxBlob =
      await Packer.toBlob(
        doc,
      );

    if (docxBlob.size > MAX_DEVICE_OUTPUT_BYTES) {
      throw new Error(
        "DOCX hasil konversi terlalu besar untuk diproses di perangkat.",
      );
    }

    onProgress?.(
      1,
      1,
      "Selesai!",
    );

    return {
      blob: docxBlob,
      filename:
        `${stem(file.name)}.docx`,
      mime:
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    };
  } finally {
    try {
      await pdf.cleanup();
    } finally {
      await task.destroy();
    }
  }
}

// ─────────────────────────────────────────────────────────────
// PDF → PowerPoint
// ─────────────────────────────────────────────────────────────

export async function pdfToPowerpoint(
  file: File,
  onProgress?: ProgressFn,
): Promise<ProcessOutput> {
  onProgress?.(
    0.1,
    1,
    "Memuat engine PDF...",
  );

  const pdfjs =
    await loadPdfjs();

  const {
    default: PptxGenJS,
  } = await loadPptxGenJs();

  const data =
    new Uint8Array(
      await file.arrayBuffer(),
    );

  if (data.byteLength > MAX_DEVICE_PDF_BYTES) {
    throw new Error(
      "PDF terlalu besar untuk diproses di perangkat (maksimum 100 MiB).",
    );
  }

  const task =
    pdfjs.getDocument({
      data,
      stopAtErrors: true,
      maxImageSize: 20_000_000,
    });

  const pdf =
    await task.promise;

  if (pdf.numPages === 0) {
    throw new Error("PDF tidak memiliki halaman.");
  }

  if (pdf.numPages > MAX_DEVICE_PDF_PAGES) {
    throw new Error(
      "PDF memiliki terlalu banyak halaman untuk diproses di perangkat (maksimum " +
        MAX_DEVICE_PDF_PAGES +
        ").",
    );
  }

  try {
    const total =
      pdf.numPages;

    const pptx =
      new PptxGenJS();

    let totalTextItems = 0;

    for (
      let pageNumber = 1;
      pageNumber <= total;
      pageNumber += 1
    ) {
      onProgress?.(
        0.1 +
          (pageNumber /
            total) *
            0.7,
        1,
        `Mengekstrak teks halaman ${pageNumber}...`,
      );

      const page =
        await pdf.getPage(
          pageNumber,
        );

      try {
        const textContent =
          await page.getTextContent();

        const items =
          getPdfTextItems(textContent.items);

        if (items.length > MAX_PDF_TEXT_ITEMS_PER_PAGE) {
          throw new Error(
            "Terlalu banyak item teks pada halaman PDF.",
          );
        }

        totalTextItems += items.length;
        if (totalTextItems > MAX_PDF_TEXT_ITEMS_TOTAL) {
          throw new Error(
            "Jumlah total item teks PDF terlalu besar untuk diproses di perangkat.",
          );
        }

        const sortedItems =
          [...items].sort(
            (a, b) => {
              const yA =
                a.transform[5];

              const yB =
                b.transform[5];

              if (
                Math.abs(
                  yA - yB,
                ) < 5
              ) {
                return (
                  a.transform[4] -
                  b.transform[4]
                );
              }

              return yB - yA;
            },
          );

        const textLines: string[] =
          [];

        let currentY = -1;
        let currentLine = "";

        for (
          const item of sortedItems
        ) {
          const y =
            item.transform[5];

          if (
            currentY === -1
          ) {
            currentY = y;
            currentLine =
              item.str;
          } else if (
            Math.abs(
              currentY - y,
            ) < 5
          ) {
            currentLine +=
              currentLine.endsWith(
                " ",
              ) ||
              item.str.startsWith(
                " ",
              )
                ? ""
                : " ";

            currentLine +=
              item.str;
          } else {
            if (
              currentLine.trim()
            ) {
              textLines.push(
                currentLine.trim(),
              );
            }

            currentY = y;
            currentLine =
              item.str;
          }
        }

        if (
          currentLine.trim()
        ) {
          textLines.push(
            currentLine.trim(),
          );
        }

        const slide =
          pptx.addSlide();

        if (
          textLines.length > 0
        ) {
          const title =
            textLines[0];

          slide.addText(
            title,
            {
              x: 0.5,
              y: 0.5,
              w: 9,
              h: 0.8,
              fontSize: 24,
              bold: true,
              color:
                "1B365D",
            },
          );

          const bodyText =
            textLines
              .slice(1)
              .join("\n");

          if (
            bodyText.trim()
          ) {
            slide.addText(
              bodyText,
              {
                x: 0.5,
                y: 1.5,
                w: 9,
                h: 4.5,
                fontSize: 14,
                color:
                  "333333",
                valign:
                  "top",
              },
            );
          }
        } else {
          slide.addText(
            `Slide ${pageNumber}`,
            {
              x: 0.5,
              y: 0.5,
              fontSize: 24,
              bold: true,
            },
          );
        }
      } finally {
        page.cleanup();
      }

      await yieldToUi();
    }

    onProgress?.(
      0.9,
      1,
      "Menghasilkan slide PowerPoint...",
    );

    const pptxBlob =
      (await pptx.write({
        outputType: "blob",
      })) as Blob;

    if (pptxBlob.size > MAX_DEVICE_OUTPUT_BYTES) {
      throw new Error(
        "PPTX hasil konversi terlalu besar untuk diproses di perangkat.",
      );
    }

    onProgress?.(
      1,
      1,
      "Selesai!",
    );

    return {
      blob: pptxBlob,
      filename:
        `${stem(file.name)}.pptx`,
      mime:
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    };
  } finally {
    try {
      await pdf.cleanup();
    } finally {
      await task.destroy();
    }
  }
}
