import { createFileRoute, Link } from "@tanstack/react-router";
import { FileCheck2, ShieldCheck, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/panduan")({
  component: Panduan,
  head: () => ({ meta: [{ title: "Panduan · pdfin" }] }),
});

function Panduan() {
  return (
    <article className="guide-page mx-auto max-w-3xl">
      <header className="guide-hero">
        <div className="guide-hero-badge">
          <ShieldCheck className="size-3.5" aria-hidden="true" />
          Panduan penggunaan
        </div>
        <p className="guide-eyebrow">pdfin</p>
        <h1>Cara pakai pdfin</h1>
        <p className="guide-hero-lede">
          Pilih alat, masukkan file, atur opsi yang tersedia, lalu proses. Alurnya dibuat sesingkat
          mungkin supaya kamu bisa fokus ke dokumen, bukan ke cara kerja aplikasinya.
        </p>
      </header>

      <section className="guide-section guide-intro-card">
        <div className="guide-section-heading">
          <span className="guide-step">01</span>
          <div>
            <h2>Mulai dari alat yang tepat</h2>
            <p>Semua alat utama bisa ditemukan dari beranda.</p>
          </div>
        </div>

        <div className="guide-feature-grid">
          <div className="guide-feature-card">
            <span className="guide-feature-icon"><Sparkles aria-hidden="true" /></span>
            <div>
              <strong>Pilih alat</strong>
              <span>Temukan fungsi untuk mengatur, mengoptimalkan, atau mengonversi dokumen.</span>
            </div>
          </div>
          <div className="guide-feature-card">
            <span className="guide-feature-icon"><FileCheck2 aria-hidden="true" /></span>
            <div>
              <strong>Periksa hasil</strong>
              <span>Setelah proses selesai, buka atau unduh hasil dan cek sebelum dibagikan.</span>
            </div>
          </div>
        </div>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">02</span>
          <div>
            <h2>Pakai alat</h2>
            <p>Alurnya sederhana dan sama untuk sebagian besar alat.</p>
          </div>
        </div>
        <ol className="guide-list">
          <li>Pilih alat dari beranda.</li>
          <li>Letakkan file ke area upload atau pilih file dari HP/laptop.</li>
          <li>Atur opsi yang tersedia sesuai kebutuhan.</li>
          <li>Tekan tombol proses dan ikuti status/progres yang ditampilkan.</li>
          <li>Setelah selesai, unduh hasilnya atau mulai lagi dengan file lain.</li>
        </ol>
        <Button asChild className="mt-6">
          <Link to="/">Lihat semua alat</Link>
        </Button>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">03</span>
          <div>
            <h2>Hal yang perlu diketahui</h2>
            <p>Batasan penting supaya ekspektasinya jelas.</p>
          </div>
        </div>
        <ul className="guide-notes">
          <li><strong>PDF berpassword</strong> belum didukung di gelombang 1.</li>
          <li><strong>File besar</strong> dapat membuat proses lebih lama. Dampaknya bergantung pada alat dan perangkat yang dipakai.</li>
          <li><strong>Kompres PDF</strong> dapat mengubah halaman menjadi gambar sehingga teks hasil kompresi tidak selalu dapat dipilih seperti PDF asli.</li>
          <li><strong>Hasil konversi</strong> tidak selalu mempertahankan layout kompleks secara persis. Gunakan hasilnya sesuai kebutuhan dan cek sebelum dibagikan.</li>
        </ul>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">04</span>
          <div>
            <h2>Tanpa instalasi untuk pengguna</h2>
            <p>Cukup buka pdfin di browser dan mulai bekerja.</p>
          </div>
        </div>
        <div className="guide-copy">
          <p>
            Kamu tidak perlu memasang TypeScript, React, Vite, atau Node.js untuk memakai situs ini.
            Semua kebutuhan penggunaan tersedia langsung di browser yang didukung.
          </p>
          <p>
            Detail teknis pengembangan merupakan urusan proyek, bukan sesuatu yang perlu kamu siapkan
            saat menggunakan pdfin.
          </p>
        </div>
      </section>

      <section className="guide-callout">
        <div className="guide-callout-icon"><ShieldCheck aria-hidden="true" /></div>
        <div>
          <h2>Kerja dengan dokumen, bukan dengan sistemnya</h2>
          <p>
            pdfin dibuat supaya informasi yang kamu perlukan muncul saat memang dibutuhkan. Fokus utama
            antarmukanya tetap pada file, pilihan yang relevan, progres, dan hasil akhir.
          </p>
        </div>
      </section>
    </article>
  );
}
