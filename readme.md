# 🖼️ IView Project

🇺🇸 A high-performance image viewer application built with Rust and egui.
🇭🇺 Egy nagy teljesítményű képnézegető alkalmazás Rust és egui alapokon.

---

## 🇺🇸 English Description

**IView** is a versatile image viewer application designed to provide efficient image management and basic editing tools, leveraging the performance and safety of the Rust ecosystem.

![IView preview](screenshots/preview.png)

### Key Features:
*   **📂 Image Browsing:** View images within a specific directory with forward/backward navigation and various sorting options.
*   **📋 Clipboard Integration:**
    *   Display images directly from the clipboard.
    *   Copy the currently displayed image to the clipboard.
*   **💾 Export & Convert:** Save loaded images in multiple formats, including `JPG`, `PNG`, `BMP`, `TIF`, `GIF`, and `WEBP`.
*   **💾 Recent path:** Bring short the recent files and path for open, and save manipulations.
*   **🎨 Image Manipulation:**
    *   **Zooming:** Scaling options ranging from 0.1x up to 10x.
    *   **Rotation:** Quick fixed-angle rotation (0°, 90°, 180°, 270°).
    *   **Adjustments:** Fine-tune Gamma, Contrast, Hue, Saturation and Brightness.
    *   **Color Tools:** Toggle individual color channels (RGB) or apply color inversion.
![IView preview](screenshots/preview_invert.jpg)
*   **⚙️ Advanced Features:**
    *   Display detailed image metadata and technical information.
    *   **Geolocation:** View stored location data directly in Google Maps.
    *   **Animation** Read, and show Webp and Gif animations.
    *   **GPU Optimization:** Automatic resizing of oversized panoramic images to the hardware-standard maximum of 16384 x 16384 pixels for stable GPU rendering.
    *   **Export with Adjustments:** Use "Save View" or "Copy View" to export the image exactly as seen on screen, including zoom levels, rotations, and color adjustments.
    *   **High-Quality Scaling:** For saving and copying, the app utilizes Lanczos3 resampling to ensure professional-grade sharpness even when resizing.

### 📖 User Guide

*   **📂 Image Management and Browsing**

    *   **Launching:** You can start the program from the command line or by clicking on its icon.
    *   **Opening:** When opened, it opens the image in the command line, or the image dragged to the shortcut, if none, the image on the clipboard, or if none, the image specified in the dialog that appears. You can also stop the program by interrupting it in the dialog. This way, the image copied in your browser can be viewed and converted immediately. 
    *   **Changing the image:** To open new images while working, use the File/Open menu item, or drag and drop an image into the window, copy from the clipboard, or navigate forward or backward through the images in the library according to the specified sorting order.

*   **🎨 Editing and Displaying**

    *   **Position:** The displayed image is either in the center of the screen or in the upper left corner. The window can be dragged, but it repositions the window when changing images.
    *   **Zoom:** You can use the slider or mouse wheel to zoom in from 0.1x to 10x. The window will expand to the maximum size of the screen, and you can move the invisible parts of the image by dragging the image or using the slider within the window.
    *   **Image correction:** Adjust Gamma, Contrast and Brightness in real time. In the Color menu, you can turn on/off the red, green and blue channels, and also set inverse colors.
    *   **Background styles:** For transparent (Png/WebP/Bmp/Tiff) images, you can choose between black, white, gray, or different checkerboard patterns in the View -> Background Style menu.
![IView preview](screenshots/preview_transparent.webp)
*   **💾 Save and Export**

    *   **Save:** It saves the original image while allowing you to switch to a different image format. In the case of Jpeg and Webp, you can also set the image quality for the save.
    *   **Copy:** The origin puts an image on the clipboard so other programs can copy it directly (rgba color model).
    *   **Save View:** Saves the image with the current changes (rotation, colors, zoom). If you are at 0.5x zoom, the image will be saved at half the size.
    *   **Copy View:** Puts the modified image on the clipboard, with pin-sharp Lanczos3 resampling.
    *   **Formats:** Supported read/save types: .jpg, .png, .webp, .tif, .bmp, .gif. For animated images, it currently reads the first image.
    *   **Restriction:** Since the interface used swallows it, the usual Ctrl+c Ctrl+v combination cannot be used. Instead, there is Alt+c, Alt+v.
    *   **GPS datas:** If the image contains geolocation metadata, a button will appear in the Info panel that will open the location directly on Google Maps.

---

## 🇭🇺 Magyar leírás

Az **IView** egy sokoldalú képnézegető alkalmazás, amely számos hasznos kiegészítő funkcióval segíti a képek kezelését és alapvető szerkesztését, kihasználva a Rust sebességét és biztonságát.

![IView preview](screenshots/preview.webp)

### Főbb funkciók:
*   **📂 Böngészés:** Képek megtekintése egy adott könyvtárban, előre-hátra léptetéssel és különböző rendezési szempontok alapján.
*   **📋 Vágólap kezelés:** 
    *   Vágólapon lévő képek közvetlen megjelenítése.
    *   A megnyitott kép vágólapra másolása.
*   **💾 Konvertálás:** Képek mentése különböző formátumokba: `JPG`, `PNG`, `BMP`, `TIF`, `GIF`, `WEBP`.
*   **💾 Legutóbbi útvonalak:** Gyors elérése a korábban használt fájlol, és útvonalaik használatára beolvasáshoz, és mentéshez.
*   **🎨 Képmódosítások:**
    *   **Nagyítás/Kicsinyítés:** Skálázható méret 0.1-től egészen 10-es szorzóig.
    *   **Forgatás:** Gyors elforgatás (0°, 90°, 180°, 270°).
    *   **Képkorrekció:** Gamma, kontraszt és világosság állítási lehetőség.
    *   **Színkezelés:** Színcsatornák (R, G, B) egyenkénti ki/be kapcsolása és inverz megjelenítés.
*   **⚙️ Speciális funkciók:**
    *   Részletes képinformációk és metaadatok megjelenítése.
    *   **Geolokáció:** Tárolt GPS koordináták megnyitása közvetlenül a Google Maps alkalmazásban.
    *   **Animáció** A Webp and Gif animációk olvasása, lejátszása.
    *   **GPU Optimalizálás:** A túl nagy panorámaképek automatikus átméretezése a grafikus processzorok (GPU) által megkövetelt maximum 16384 x 16384 képpontos méretre.
    *   **Módosítások exportálása:** Lehetőség van a képernyőn látható módosítások (nagyítás/kicsinyítés, forgatás, LUT effektek) alkalmazásával menteni a képet ("Save View") vagy a vágólapra másolni azt ("Copy View").
    *   **Prémium átméretezés:** Mentésnél és másolásnál az alkalmazás Lanczos3 mintavételezést használ, ami tűéles minőséget biztosít kicsinyítés esetén is.
---
### 📖 Használati útmutató

*   **📂 Képkezelés és Böngészés**

    *   **Indítás:** A programot indíthatod parancssorból, vagy az ikonjára kattintva.
    *   **Megnyitás:** Megnyitáskor a parancssorban levő képet, vagy a parancsikonra húzott képet, ennek hiányában a vágólapon levő képet, ennek hiányában a feljövő dialógban megadott képet nyitja meg. A dialógban való megszakítással le is állíthatod a programot. Így a böngésződben másolt kép azonnal megnézhető, és átalakítható. 
    *   **A kép váltása:** Menet közbeni újabb képek megnyitására használd a File/Open menüpontot, vagy húzz be egy képet az ablakba (Drag & Drop), vagy a vágólapról másolj, vagy navigálj a könyvtárban levő képeken előre, vagy hátra a megadott rendezési sorrend szerint.

*   **🎨 Szerkesztés és Megjelenítés**

    *   **Pozíció:** A megjelenített kép vagy a képernyő közepén, vagy a bal felső sarokban jelenik meg. Az ablak elhúzható, de képváltáskor újra pozicionálja az ablakot.
    *   **Nagyítás:** A csúszkával vagy egérgörgővel 0.1x és 10x közötti mérettartományt érhetsz el. Az ablak maximum a képernyő nagyságáig növekszik, a nem látható részeket a kép húzásával, vagy a csúszkával mozgathatjuk az ablakon belül.
    *   **Képkorrekció:** Állítsd a Gammát, Kontrasztot és Világosságot valós időben. A Color menüben ki/be kapcsolhatod a piros, zöld és kék csatornákat, inverz színeket is beállíthatsz.
    *   **Háttérstílusok:** Átlátszó (Png/WebP/Bmp/Tiff) képek esetén a View -> Background Style menüben választhatsz fekete, fehér, szürke vagy a különböző sakktábla minták között.

*   **💾 Mentés és Exportálás**

    *   **Save:** Elmenti az eredeti képet, miközben más kép formátumra válthatsz. Jpeg és Webp esetén a mentés képminőségét is beállíthatod.
    *   **Copy:** Az eredet képet teszi a vágólapra, így más programok közvetlenül átvehetik azt (rgba színmodell).
    *   **Save View:** Elmenti a képet a jelenlegi módosításokkal (forgatás, színek, nagyítás). Ha 0.5x nagyításon állsz, a kép feleakkora méretben kerül mentésre.
    *   **Copy View:** A módosított képet teszi a vágólapra, tűéles Lanczos3 újramintavételezéssel.
    *   **Formátumok:** Támogatott olvasási/mentési típusok: .jpg, .png, .webp, .tif, .bmp, .gif. Animált képeknél jelenleg az első képet olvassa.
    *   **Korlátozás:** Mivel a használt interfész lenyeli, így nem használható a szokásos Ctrl+c Ctrl+v kombináció. Helyette Alt+c, Alt+v van.
    *   **GPS adatok:** Ha a kép tartalmaz geolokációs metaadatokat, az Info panelen megjelenik egy gomb, amellyel a helyszín közvetlenül megnyitható a Google Maps-en.

---

*   **⌨️ Gyorsbillentyűk / Shortcuts**


| Key | Function |
| --- | --- |
| B / N | Before / Next image in directory |
| O | Open image |
| R | Reopen same image (hide/show inside/outside modification)|
| S | Save image  & convert to other type) |
| Shift + S | Save modified view & convert |
| Alt + C | Copy |
| Alt + Shift + C | Copy View  |
| Alt + V | Paste  |
| Escape | exit from popup windows or program  |
| Ctrl + R | Toggle red channel |
| Ctrl + G | Toggle greeen channel |
| Ctrl + B | Toggle blue channel |
| Ctrl + I | Invert color channels |
| C | Open color corrections window |
| I | Open informations window |
| D | Toggle backgrounds for transparent images |

---
### 🛠 Tech Stack / Technológiai háttér

*   **Language:** [Rust](https://www.rust-lang.org)
*   **UI Framework:** [eframe] / [egui] 

*   **Cross-platform:** Tested and working on Windows 10 and Linux (Linux Mint).

*   **Executables:** in the executables folder
*   **Latest Version:** 0.2.0 

### 🚀 Development / Fejlesztés

```bash
# Build and run the project
# Projekt fordítása és futtatása
git clone https://github.com/Ferenc-Takacs/IView.git
cd IView
cargo run --release
```
![IView preview](screenshots/preview.jpg)