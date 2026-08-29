import { createFileRoute, Link } from "@tanstack/react-router";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/panduan")({
  component: Panduan,
  head: () => ({ meta: [{ title: "Panduan · pdfin" }] }),
});

function Panduan() {
  return (
    <article className="mx-auto max-w-2xl">
      <p className="text-xs font-medium uppercase tracking-wider text-primary">Panduan</p>
      <h1 className="mt-1 text-3xl font-semibold tracking-tight sm:text-4xl">Cara pakai pdfin</h1>
      <p className="mt-3 text-muted">
        Situs ini sudah hidup di pratinjau. Tidak perlu install TypeScript, Node, atau program
        PDF di komputer untuk memakai alatnya.
      </p>

      <section className="mt-10">
        <h2 className="text-xl font-semibold">1. Pakai sekarang</h2>
        <ol className="mt-4 list-decimal space-y-3 pl-5 text-sm leading-relaxed text-fg">
          <li>Pilih alat di beranda, misalnya Gabung atau Kompres.</li>
          <li>Letakkan file ke kotak, atau ketuk untuk memilih dari HP/laptop.</li>
          <li>Atur opsi (rentang halaman, kualitas, teks stempel).</li>
          <li>Ketuk <strong>Proses di perangkat ini</strong>. Tunggu bar progres.</li>
          <li>Unduh hasilnya. File asli tidak diubah.</li>
        </ol>
        <Button asChild className="mt-5">
          <Link to="/">Lihat semua alat</Link>
        </Button>
      </section>

      <section className="mt-10">
        <h2 className="text-xl font-semibold">2. Yang perlu diketahui</h2>
        <ul className="mt-4 space-y-3 text-sm leading-relaxed text-muted">
          <li>
            <span className="font-medium text-fg">Tidak ada unggahan.</span> File diproses di
            Chrome/Edge/Safari kamu. Tutup tab, data hilang dari memori.
          </li>
          <li>
            <span className="font-medium text-fg">PDF terkunci password</span> belum bisa dibuka
            di gelombang 1.
          </li>
          <li>
            <span className="font-medium text-fg">File sangat besar</span> (puluhan MB, ratusan
            halaman) bisa membuat HP terasa berat  -  itu perangkatmu yang kerja, bukan server.
          </li>
          <li>
            <span className="font-medium text-fg">Kompres</span> mengubah halaman jadi gambar
            agar ukurannya turun. Teks tidak bisa diseleksi lagi. Itu wajar untuk kirim chat.
          </li>
        </ul>
      </section>

      <section className="mt-10">
        <h2 className="text-xl font-semibold">3. TypeScript tidak diinstall terpisah</h2>
        <p className="mt-3 text-sm leading-relaxed text-muted">
          TypeScript sudah termasuk di dalam proyek web ini, sama seperti mesin di dalam mobil  - 
          kamu tidak “install TypeScript” di Windows seperti install Word.
        </p>
        <p className="mt-3 text-sm leading-relaxed text-muted">
          Untuk <em>memakai</em> pdfin: buka situs, selesai. Nol program tambahan.
        </p>
        <p className="mt-3 text-sm leading-relaxed text-muted">
          Untuk <em>mengembangkan sendiri nanti</em> di PC i5, yang perlu hanya satu program:
          <strong className="text-fg"> Node.js LTS</strong> (versi 22). Setelah itu, di folder
          proyek: pasang dependensi, jalankan mode pengembangan. TypeScript, React, dan Vite
          ikut terpasang otomatis dari daftar proyek  -  bukan diinstall satu-satu.
        </p>
      </section>

      <section className="mt-10 rounded-2xl bg-surface p-5 shadow-(--shadow-card)">
        <h2 className="text-lg font-semibold">Nanti, kalau mau jalan di PC sendiri</h2>
        <ol className="mt-3 list-decimal space-y-2 pl-5 text-sm text-muted">
          <li>Install Node.js LTS dari situs resmi Node (bukan TypeScript terpisah).</li>
          <li>Buka folder proyek pdfin.</li>
          <li>
            Di terminal: <code className="rounded bg-bg px-1.5 py-0.5 text-fg">npm install</code>{" "}
            sekali, lalu{" "}
            <code className="rounded bg-bg px-1.5 py-0.5 text-fg">npm run dev</code>.
          </li>
          <li>Browser akan membuka situs lokal. PC tidak perlu nyala 24 jam untuk pengunjung.</li>
        </ol>
        <p className="mt-3 text-sm text-muted">
          Untuk publik: unggah hasil build ke Vercel atau Cloudflare Pages. Pengunjung membuka
          alamat web, PC kamu boleh mati.
        </p>
      </section>
    </article>
  );
}
