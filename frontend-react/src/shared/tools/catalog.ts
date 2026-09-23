export type ToolSlug =
  | "gabung"
  | "pisah"
  | "halaman"
  | "kompres"
  | "jpg-ke-pdf"
  | "pdf-ke-jpg"
  | "putar"
  | "watermark"
  | "word-ke-pdf"
  | "excel-ke-pdf"
  | "powerpoint-ke-pdf"
  | "pdf-ke-word"
  | "pdf-ke-excel"
;

export type AcceptKind = "pdf" | "image" | "word" | "excel" | "powerpoint";
export type ProcessingLocation = "device" | "server";

export type ToolDef = {
  slug: ToolSlug;
  title: string;
  short: string;
  description: string;
  hint: string;
  accept: AcceptKind;
  multiple: boolean;
  minFiles: number;
  group: "atur" | "optimalkan" | "konversi";
  processing: ProcessingLocation;
};

export const TOOLS: ToolDef[] = [
  {
    slug: "gabung",
    title: "Gabung PDF",
    short: "Gabung",
    description: "Satukan beberapa PDF jadi satu berkas, urutan sesuai daftar.",
    hint: "Cocok untuk KTP + NPWP, lampiran lamaran, atau banyak scan jadi satu.",
    accept: "pdf",
    multiple: true,
    minFiles: 2,
    group: "atur",
    processing: "server",
  },
  {
    slug: "pisah",
    title: "Pisah PDF",
    short: "Pisah",
    description: "Ambil rentang halaman, atau pecah tiap halaman jadi file sendiri.",
    hint: "Contoh rentang: 1-3, 5, 8-10",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "atur",
    processing: "server",
  },
  {
    slug: "halaman",
    title: "Atur halaman",
    short: "Halaman",
    description: "Urutkan, hapus, atau susun ulang halaman lewat pratinjau.",
    hint: "Satu PDF. Geser urutan, buang halaman yang tidak perlu, lalu unduh.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "atur",
    processing: "server",
  },
  {
    slug: "putar",
    title: "Putar PDF",
    short: "Putar",
    description: "Putar semua halaman 90°, 180°, atau 270°.",
    hint: "Untuk scan yang terbalik atau foto dokumen miring.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "atur",
    processing: "server",
  },
  {
    slug: "kompres",
    title: "Kompres PDF",
    short: "Kompres",
    description: "Kecilkan ukuran agar muat WhatsApp, email, atau formulir online.",
    hint: "Pilih 'Kecil' untuk kirim lewat chat.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "optimalkan",
    processing: "server",
  },
  {
    slug: "watermark",
    title: "Watermark & nomor",
    short: "Watermark",
    description: "Stempel teks (ASLI, DRAFT, RAHASIA) dan nomor halaman.",
    hint: "Teks miring di tengah + nomor di kaki halaman, opsional.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "optimalkan",
    processing: "device",
  },
  {
    slug: "jpg-ke-pdf",
    title: "JPG ke PDF",
    short: "JPG → PDF",
    description: "Ubah foto atau scan jadi satu PDF rapi ukuran A4.",
    hint: "JPG, PNG, atau WebP. Beberapa gambar = beberapa halaman.",
    accept: "image",
    multiple: true,
    minFiles: 1,
    group: "konversi",
    processing: "server",
  },
  {
    slug: "pdf-ke-jpg",
    title: "PDF ke JPG",
    short: "PDF → JPG",
    description: "Ubah tiap halaman PDF menjadi gambar JPG.",
    hint: "Satu halaman = satu JPG. Banyak halaman diunduh sebagai ZIP.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "device",
  },
  {
    slug: "word-ke-pdf",
    title: "Word ke PDF",
    short: "Word → PDF",
    description: "Ubah dokumen Word (.docx) menjadi PDF.",
    hint: "Mendukung .docx. Layout sederhana (teks, heading, list).",
    accept: "word",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "device",
  },
  {
    slug: "excel-ke-pdf",
    title: "Excel ke PDF",
    short: "Excel → PDF",
    description: "Ubah spreadsheet Excel (.xlsx) menjadi PDF.",
    hint: "Sheet pertama diekspor sebagai tabel di PDF. Cocok untuk data tabulasi.",
    accept: "excel",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "server",
  },
  {
    slug: "powerpoint-ke-pdf",
    title: "PowerPoint ke PDF",
    short: "PowerPoint → PDF",
    description: "Ubah presentasi PowerPoint (.pptx) menjadi PDF.",
    hint: "Teks slide diekspor ke PDF. Cocok untuk presentasi sederhana.",
    accept: "powerpoint",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "device",
  },
  {
    slug: "pdf-ke-word",
    title: "PDF ke Word",
    short: "PDF → Word",
    description: "Ekstrak teks PDF menjadi dokumen Word (.docx) yang bisa diedit.",
    hint: "Hasil terbaik untuk PDF berteks (bukan scan). Layout tidak selalu sama persis.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "device",
  },
  {
    slug: "pdf-ke-excel",
    title: "PDF ke Excel",
    short: "PDF → Excel",
    description: "Ekstrak teks/baris dari PDF ke file Excel (.xlsx).",
    hint: "Cocok untuk PDF berisi daftar atau tabel sederhana. Bukan OCR scan.",
    accept: "pdf",
    multiple: false,
    minFiles: 1,
    group: "konversi",
    processing: "server",
  },

];

export const GROUPS: { id: ToolDef["group"]; label: string }[] = [
  { id: "atur", label: "Atur PDF" },
  { id: "optimalkan", label: "Optimalkan" },
  { id: "konversi", label: "Konversi" },
];

export function getTool(slug: string | undefined): ToolDef | undefined {
  return TOOLS.find((t) => t.slug === slug);
}

export const PDF_ACCEPT = "application/pdf,.pdf";
export const IMAGE_ACCEPT =
  "image/jpeg,image/png,image/webp,image/jpg,.jpg,.jpeg,.png,.webp";
export const WORD_ACCEPT =
  "application/vnd.openxmlformats-officedocument.wordprocessingml.document,.docx";
export const EXCEL_ACCEPT =
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet,.xlsx";
export const POWERPOINT_ACCEPT =
  "application/vnd.openxmlformats-officedocument.presentationml.presentation,.pptx";

export function acceptFor(kind: AcceptKind): string {
  switch (kind) {
    case "pdf":
      return PDF_ACCEPT;
    case "image":
      return IMAGE_ACCEPT;
    case "word":
      return WORD_ACCEPT;
    case "excel":
      return EXCEL_ACCEPT;
    case "powerpoint":
      return POWERPOINT_ACCEPT;
  }
}
