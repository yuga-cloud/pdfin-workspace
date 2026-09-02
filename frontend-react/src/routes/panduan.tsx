import { createFileRoute, Link } from "@tanstack/react-router";
import { CloudCog, MonitorCheck, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { TOOLS } from "@/lib/tools-catalog";

export const Route = createFileRoute("/panduan")({
  component: Panduan,
  head: () => ({ meta: [{ title: "Panduan · pdfin" }] }),
});

function Panduan() {
  const deviceCount = TOOLS.filter((tool) => tool.processing === "device").length;
  const serverCount = TOOLS.filter((tool) => tool.processing === "server").length;

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
          Pilih alat, masukkan file, atur opsi yang tersedia, lalu proses. Setiap alat memberi tahu
          di awal apakah file diproses di perangkat atau dikirim ke server.
        </p>
      </header>

      <section className="guide-section guide-processing-card">
        <div className="guide-section-heading">
          <span className="guide-step">01</span>
          <div>
            <h2>Kenali lokasi pemrosesan</h2>
            <p>Ini berbeda menurut alat yang kamu pilih.</p>
          </div>
        </div>

        <div className="processing-mode-grid">
          <div className="processing-mode-card device">
            <span className="processing-mode-icon"><MonitorCheck aria-hidden="true" /></span>
            <div>
              <strong>Di perangkat</strong>
              <span>{deviceCount} alat saat ini berjalan di browser kamu.</span>
            </div>
          </div>
          <div className="processing-mode-card server">
            <span className="processing-mode-icon"><CloudCog aria-hidden="true" /></span>
            <div>
              <strong>Di server</strong>
              <span>{serverCount} alat saat ini mengirim file ke server untuk diproses.</span>
            </div>
          </div>
        </div>

        <p className="guide-callout-note">
          Tidak ada janji privasi yang disembunyikan di balik satu kalimat umum. Buka alat yang akan
          kamu gunakan dan baca label pemrosesannya sebelum memilih file.
        </p>
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
          <li>Baca label lokasi pemrosesan dan atur opsi yang tersedia.</li>
          <li>Tekan tombol proses dan ikuti status/progres yang ditampilkan.</li>
          <li>Setelah selesai, unduh hasilnya atau proses file lain.</li>
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
            <h2>Tidak perlu install apa pun untuk memakai situs</h2>
            <p>Setup developer hanya diperlukan untuk mengembangkan aplikasinya.</p>
          </div>
        </div>
        <div className="guide-copy">
          <p>
            Untuk memakai pdfin, cukup buka situsnya di browser yang didukung. Kamu tidak perlu
            memasang TypeScript, React, Vite, atau Node.js.
          </p>
          <p>
            Untuk mengembangkan pdfin sendiri, proyek web sudah menyertakan kebutuhan TypeScript,
            React, dan Vite sebagai dependensi. Jalankan setup developer yang tercantum di proyek;
            jangan memasang tool satu per satu secara manual.
          </p>
        </div>
      </section>

      <section className="guide-callout">
        <div className="guide-callout-icon"><ShieldCheck aria-hidden="true" /></div>
        <div>
          <h2>Prinsip pdfin</h2>
          <p>
            Kami lebih memilih menjelaskan bagaimana sebuah alat bekerja daripada membuat klaim
            privasi yang terdengar lebih aman dari kenyataannya. Lokasi pemrosesan selalu ditampilkan
            sesuai konfigurasi alat saat ini.
          </p>
        </div>
      </section>
    </article>
  );
}
