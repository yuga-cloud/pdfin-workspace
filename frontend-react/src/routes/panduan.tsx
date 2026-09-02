import { createFileRoute, Link } from "@tanstack/react-router";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/panduan")({
  component: Panduan,
  head: () => ({ meta: [{ title: "Panduan · pdfin" }] }),
});

function Panduan() {
  return (
    <article className="guide-page mx-auto max-w-3xl">
      <header className="guide-hero">
        <p className="guide-eyebrow text-xs font-bold uppercase tracking-[0.16em] text-primary">Panduan</p>
        <h1 className="mt-2 text-3xl font-semibold tracking-[-0.035em] sm:text-5xl">Cara pakai pdfin</h1>
        <p className="mt-4 max-w-2xl text-base leading-relaxed text-muted sm:text-lg">
          Pilih alat, masukkan file, periksa lokasi pemrosesannya, lalu unduh hasilnya. Tidak perlu
          install program tambahan untuk memakai pdfin.
        </p>
      </header>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">01</span>
          <div>
            <h2 className="text-xl font-semibold tracking-tight">Pakai sekarang</h2>
            <p className="mt-1 text-sm text-muted">Alur dasarnya hanya beberapa langkah.</p>
          </div>
        </div>
        <ol className="guide-list mt-5 list-decimal space-y-3 pl-5 text-sm leading-relaxed text-fg">
          <li>Pilih alat di beranda, misalnya Gabung atau Kompres.</li>
          <li>Letakkan file ke kotak, atau ketuk untuk memilih dari HP/laptop.</li>
          <li>Perhatikan keterangan lokasi pemrosesan pada alat yang dipilih.</li>
          <li>Atur opsi yang tersedia, lalu jalankan prosesnya.</li>
          <li>Unduh hasilnya. File asli tidak diubah.</li>
        </ol>
        <Button asChild className="mt-6">
          <Link to="/">Lihat semua alat</Link>
        </Button>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">02</span>
          <div>
            <h2 className="text-xl font-semibold tracking-tight">Lokasi pemrosesan</h2>
            <p className="mt-1 text-sm text-muted">Tidak semua alat bekerja dengan cara yang sama.</p>
          </div>
        </div>
        <ul className="guide-notes mt-5 space-y-3 text-sm leading-relaxed text-muted">
          <li>
            <span className="font-semibold text-fg">Diproses di perangkat.</span> File dikerjakan di browser pada HP atau laptop kamu dan tidak perlu dikirim ke server untuk proses tersebut.
          </li>
          <li>
            <span className="font-semibold text-fg">Diproses di server.</span> File dikirim ke server agar pemrosesan alat tersebut dapat dilakukan. Keterangan ini ditampilkan pada halaman alat.
          </li>
          <li>
            <span className="font-semibold text-fg">Periksa sebelum memilih.</span> Lokasi pemrosesan tercantum pada setiap alat agar kamu bisa memilih sesuai kebutuhan privasi dan perangkat.
          </li>
          <li>
            <span className="font-semibold text-fg">File besar</span> bisa membutuhkan waktu lebih lama atau terasa berat. Dampaknya bergantung pada alat dan lokasi pemrosesannya.
          </li>
        </ul>
      </section>

      <section className="guide-section">
        <div className="guide-section-heading">
          <span className="guide-step">03</span>
          <div>
            <h2 className="text-xl font-semibold tracking-tight">Tentang pdfin</h2>
            <p className="mt-1 text-sm text-muted">Yang perlu diketahui sebelum menggunakan alat.</p>
          </div>
        </div>
        <div className="mt-5 space-y-3 text-sm leading-relaxed text-muted">
          <p>
            Untuk memakai pdfin, kamu tidak perlu menginstal TypeScript, Node.js, atau program PDF tambahan. Semua kebutuhan untuk pengguna biasa sudah berjalan dari situs.
          </p>
          <p>
            Beberapa alat memang berjalan langsung di browser, sementara alat lain membutuhkan server. Kami menampilkan lokasi pemrosesan pada halaman alat agar informasi tersebut jelas sebelum kamu mengirim file.
          </p>
          <p>
            Hasil konversi atau perubahan layout dapat berbeda menurut jenis dokumen. Untuk file kompleks, selalu periksa hasil sebelum digunakan lebih lanjut.
          </p>
        </div>
      </section>

      <section className="guide-callout rounded-2xl bg-surface p-5 shadow-(--shadow-card) sm:p-6">
        <h2 className="text-lg font-semibold tracking-tight">Untuk pengembang</h2>
        <ol className="mt-4 list-decimal space-y-2 pl-5 text-sm leading-relaxed text-muted">
          <li>Gunakan Node.js LTS untuk lingkungan pengembangan frontend.</li>
          <li>Buka folder proyek pdfin.</li>
          <li>
            Jalankan <code className="rounded-lg border border-border bg-bg px-1.5 py-0.5 text-fg">npm install</code>{" "}
            sekali, lalu{" "}
            <code className="rounded-lg border border-border bg-bg px-1.5 py-0.5 text-fg">npm run dev</code>.
          </li>
          <li>Browser akan membuka aplikasi frontend lokal.</li>
        </ol>
      </section>
    </article>
  );
}
