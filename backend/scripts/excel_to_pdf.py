import sys
import time
import urllib.request

import uno  # type: ignore
from com.sun.star.beans import PropertyValue  # type: ignore
from com.sun.star.connection import NoConnectException  # type: ignore


def main():
    input_url = "file:" + urllib.request.pathname2url(sys.argv[1])
    output_url = "file:" + urllib.request.pathname2url(sys.argv[2])
    pipe_name = sys.argv[3]

    local_ctx = uno.getComponentContext()
    resolver = local_ctx.ServiceManager.createInstanceWithContext(
        "com.sun.star.bridge.UnoUrlResolver", local_ctx
    )

    # Mekanisme retry: Coba hubungkan ke pipe selama beberapa detik karena LibreOffice butuh waktu startup
    ctx = None
    for _ in range(40):  # Mencoba hingga 40 kali (total ~4 detik)
        try:
            ctx = resolver.resolve(
                "uno:pipe,name=" + pipe_name + ";urp;StarOffice.ComponentContext"
            )
            break
        except NoConnectException:
            time.sleep(0.1)

    if ctx is None:
        raise RuntimeError(
            f"Gagal terhubung ke LibreOffice pipe '{pipe_name}' setelah beberapa percobaan."
        )

    desktop = ctx.ServiceManager.createInstanceWithContext(
        "com.sun.star.frame.Desktop", ctx
    )

    in_props = (PropertyValue(Name="Hidden", Value=True),)
    doc = desktop.loadComponentFromURL(input_url, "_blank", 0, in_props)

    if doc.supportsService("com.sun.star.sheet.SpreadsheetDocument"):
        styles = doc.getStyleFamilies().getByName("PageStyles")
        for i in range(doc.getSheets().getCount()):
            sheet = doc.getSheets().getByIndex(i)
            style = styles.getByName(sheet.PageStyle)
            style.IsLandscape = True
            style.ScaleToPagesX = 1
            style.ScaleToPagesY = 0

    out_props = (PropertyValue(Name="FilterName", Value="calc_pdf_Export"),)
    doc.storeToURL(output_url, out_props)
    doc.close(True)


if __name__ == "__main__":
    main()
