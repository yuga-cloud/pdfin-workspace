import { Link } from "@tanstack/react-router";
import { ArrowRight, CheckCircle2, FileCheck2, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Panduan() {
  return (
    <article className="guide-page mx-auto max-w-3xl">
      <header className="guide-hero">
        <p className="guide-eyebrow">Panduan pdfin</p>
        <h1>Cara pakai pdfin</h1>
        <p className="guide-hero-lede">
          Pilih alat, masukkan file, sesuaikan pilihan yang tersedia, lalu unduh hasilnya.
          Semua dibuat supaya alur kerja tetap sederhana.
        </p>
      </header>

      <section className="guide-section guide-intro-card">
        <div className="guide-section-heading">
          <span className="guide-step">01</span>
          <div>
            <h2>Mulai dari alat yang sesuai</h2>
            <p>Pilih berdasarkan pekerjaan yang sedang kamu lakukan.</p>
          </div>
        </div>

        <div className="guide-feature-grid">
          <div className="guide-feature-card">
            <span className="guide-feature-icon"><Sparkles aria-hidden="true" /></span>
            <div>
              <strong>Atur PDF</strong>
              <span>Gabung, pisah, putar, atau susun ulang halaman.</span>
            </div>
          </div>
          <div className="guide-feature-card">
            <span className="guide-feature-icon"><FileCheck2 aria-hidden="true" /></span>
            <div>
              <strong>Optimalkan & konversi</strong>
              <span>Kecilkan file atau ubah dokumen ke format yang dibutuhkan.</span>
            </div>
          </div>
        </div>

        <Button asChild className="guide-primary-action">
          <Link to="/">
            Lihat semua alat
            <ArrowRight aria-hidden="true" />
          </Link>
        </Button>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">02</span>
          <div>
            <h2>Ikuti alurnya</h2>
            <p>Langkahnya hampir selalu sama.</p>
          </div>
        </div>
        <ol className="guide-list">
          <li>Pilih alat yang kamu perlukan.</li>
          <li>Masukkan file dengan menyeretnya ke area upload atau memilihnya dari perangkat.</li>
          <li>Sesuaikan opsi yang tersedia.</li>
          <li>Tekan <strong>Proses</strong> dan tunggu sampai selesai.</li>
          <li>Unduh hasilnya, atau mulai lagi dengan file lain.</li>
        </ol>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">03</span>
          <div>
            <h2>Supaya hasilnya sesuai</h2>
            <p>Beberapa hal layak diperiksa sebelum file dibagikan.</p>
          </div>
        </div>
        <ul className="guide-notes">
          <li><strong>Periksa hasil akhir</strong> terutama untuk dokumen dengan layout kompleks.</li>
          <li><strong>File besar</strong> bisa membutuhkan waktu lebih lama untuk diproses.</li>
          <li><strong>PDF berpassword</strong> belum didukung pada gelombang awal.</li>
          <li><strong>Kompresi</strong> dapat mengurangi ukuran dengan konsekuensi pada kualitas atau kemampuan memilih teks, tergantung dokumen.</li>
        </ul>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">04</span>
          <div>
            <h2>Tips cepat</h2>
            <p>Gunakan alur yang paling ringan untuk pekerjaanmu.</p>
          </div>
        </div>
        <div className="guide-check-list">
          <div><CheckCircle2 aria-hidden="true" /><span>Gunakan nama file yang jelas sebelum upload.</span></div>
          <div><CheckCircle2 aria-hidden="true" /><span>Untuk banyak dokumen, rapikan urutan file terlebih dahulu.</span></div>
          <div><CheckCircle2 aria-hidden="true" /><span>Setelah proses selesai, cek nama file dan ukuran hasil sebelum dibagikan.</span></div>
        </div>
      </section>

      <section className="guide-callout">
        <div className="guide-callout-icon"><CheckCircle2 aria-hidden="true" /></div>
        <div>
          <h2>Fokus ke dokumennya</h2>
          <p>
            pdfin dibuat agar kamu tidak perlu memikirkan cara kerja di balik layar.
            Pilih alat, kerjakan file, lalu ambil hasilnya.
          </p>
        </div>
      </section>
    </article>
  );
}
