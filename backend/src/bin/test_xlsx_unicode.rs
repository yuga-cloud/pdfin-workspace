use rust_xlsxwriter::Workbook;

fn main() {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    worksheet
        .write_string(0, 0, "Nama")
        .expect("Gagal menulis Nama");

    worksheet
        .write_string(1, 0, "Nilai")
        .expect("Gagal menulis Nilai");

    worksheet
        .write_string(2, 0, "PDFin Excel to PDF Test")
        .expect("Gagal menulis text");

    worksheet
        .write_string(3, 0, "Indonesia é à ü 中文 日本語")
        .expect("Gagal menulis Unicode");

    workbook
        .save("/home/sync/pdfin-test/test-unicode.xlsx")
        .expect("Gagal menyimpan XLSX");
}
