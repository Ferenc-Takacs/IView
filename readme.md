# 🖼️ IView Project

🇺🇸 A high-performance image viewer application built with Rust and egui.
🇭🇺 Egy nagy teljesítményű képnézegető alkalmazás Rust és egui alapokon.

---

## 🇺🇸 English Description

**IView** is a versatile image viewer application designed to provide efficient image management and basic editing tools, leveraging the performance and safety of the Rust ecosystem.  It takes advantage of the computing power of the graphics card, or uses the CPU on virtual machines that do not provide it.

![IView preview](screenshots/preview.png)

### Key Features:
*   **📂 Image Browsing:** View images within a specific directory with forward/backward navigation and various sorting options.
*   **📋 Clipboard Integration:**
    *   Display images directly from the clipboard.
    *   Copy the currently displayed image to the clipboard.
    *   Replace the opened image with the image on the clipboard.
*   **💾 Export & Convert:** Save loaded images in multiple formats, including `JPG`, `PNG`, `BMP`, `TIF`, `GIF`, and `WEBP`.
*   **💾 Recent path:** Quick access to previously used files and their paths for reading and saving.
*   **🎨 Image Manipulation:**
    *   **Zooming:** Scaling options ranging from 0.1x up to 10x.
    *   **Rotation:** Quick fixed-angle rotation (0°, 90°, 180°, 270°).
    *   **Adjustments:** Fine-tune Gamma, Contrast, Hue, Saturation and Brightness, Gaussian Blur/Sharpen, color rotation in Oklab or Hsv color space, color saturation adjustment.
    *   **Transparent color:** Designates a given color and its surroundings as a transparent color.
    *   **Color Tools:** Toggle individual color channels (RGB) or apply color inversion.

![IView preview](screenshots/preview_invert.jpg)

*   **⚙️ Advanced Features:**
    *   Display detailed image metadata and technical information.
    *   **Geolocation:** View stored location data directly in Google Maps.
    *   **Animation** Read, and show Webp and Gif animations.
    *   **Histogram** Show the frequency of occurrence of each color.
    *   **PickPixel** Info about the position and color of a given point in the image.
    *   **GPU Optimization:** Automatic resizing of oversized panoramic images to the hardware-standard maximum of 16384 x 16384 pixels for stable GPU rendering.
    *   **Export with Adjustments:** Use "Save View" or "Copy View" to export the image exactly as seen on screen, including zoom levels, rotations, and color adjustments.
    *   **High-Quality Scaling:** For saving and copying, the app utilizes Lanczos3 resampling to ensure professional-grade sharpness even when resizing.

### 📖 User Guide

*   **📂 Image Management and Browsing**

    *   **Launching:** You can start the program from the command line or by clicking on its icon.
    *   **Opening:** When opened, it opens the image in the command line, or the image dragged to the shortcut, if none, the image on the clipboard, or if none, the image specified in the dialog that appears.This way, the image copied in your browser can be viewed and converted immediately. You can also stop the program by canceling in the dialog and can choosing from previously used images.
    *   **Changing the image:** To open new images while working, use the File/Open menu item, or drag and drop an image into the window, copy from the clipboard, or navigate forward or backward through the images in the library according to the specified sorting order.

*   **🎨 Editing and Displaying**

    *   **Position:** The displayed image is either in the center of the screen or in the upper left corner. The window can be dragged, but it repositions the window when changing images.
    *   **Zoom:** You can use the slider or mouse wheel to zoom in from 0.1x to 10x. The window will expand to the maximum size of the screen, and you can move the invisible parts of the image by dragging the image or using the slider within the window.
    *   **Image correction:** Adjust Gamma, Contrast and Brightness in real time. In the Color menu, you can turn on/off the red, green and blue channels, and also set inverse colors. You can use the Blur/Sharpen functions to blur or sharpen boundaries, or make a range of colors transparent.
    *   **Background styles:** For transparent (Png/WebP/Bmp/Tiff) images, you can choose between black, white, gray, or different checkerboard patterns in the View -> Background Style menu.
	
![IView preview](screenshots/preview_transparent.webp)

*   **💾 Save and Export**

    *   **Save:** It saves the original image while allowing you to switch to a different image format. In the case of Jpeg and Webp, you can also set the image quality for the save.
    *   **Save View:** Saves the image with the current changes (rotation, colors, zoom). If you are at 0.5x zoom, the image will be saved at half the size.
    *   **Copy:** The origin puts an image on the clipboard so other programs can copy it directly (rgba color model).
    *   **Copy View:** Puts the modified image on the clipboard, with pin-sharp Lanczos3 resampling.
    *   **Paste:** Imports the image from the clipboard into the program.
    *   **Change:** It places the original image on the clipboard while importing the image there into the program.
    *   **Change View:** It places the modified image on the clipboard while importing the image that is there. This allows you to repeat the modifications.
    *   **Formats:** Supported read/save types: .jpg, .png, .webp, .tif, .bmp, .gif. For animated images, it currently reads the first image.
    *   **Restriction:** The Ctrl + c,v,x functions work when the button is released due to a limitation of the egui system.
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
    *   A megnyitott kép felcserélése a vágólapon levő képpel.
*   **💾 Konvertálás:** Képek mentése különböző formátumokba: `JPG`, `PNG`, `BMP`, `TIF`, `GIF`, `WEBP`.
*   **💾 Legutóbbi útvonalak:** Gyors elérése a korábban használt fájlok, és útvonalaik használatára beolvasáshoz, és mentéshez.
*   **🎨 Képmódosítások:**
    *   **Nagyítás/Kicsinyítés:** Skálázható méret 0.1-től egészen 10-es szorzóig.
    *   **Forgatás:** Gyors elforgatás (0°, 90°, 180°, 270°).
    *   **Képkorrekció:** Gamma, kontraszt és világosság állítási lehetőség, Gaussian élesítés/homályosítás, színforgatás az Oklab vagy Hsv színtérben, színtelítettség állítás.
    *   **Átlátszó szín:** Adott szín, és környezete kijelölése átlátszó színnek.
    *   **Színkezelés:** Színcsatornák (R, G, B) egyenkénti ki/be kapcsolása és inverz megjelenítés.
*   **⚙️ Speciális funkciók:**
    *   Részletes képinformációk és metaadatok megjelenítése.
    *   **Geolokáció:** Tárolt GPS koordináták megnyitása közvetlenül a Google Maps alkalmazásban.
    *   **Animáció** A Webp and Gif animációk olvasása, lejátszása, írása képként vagy egészben.
    *   **Hisztogram** Az egyes színek előfordulási gyakoriságának megjelenítése.
    *   **PickPixel** Info a kép adott pontja pozíciójáról, és színéről.
    *   **GPU Optimalizálás:** A túl nagy panorámaképek automatikus átméretezése a grafikus processzorok (GPU) által megkövetelt maximum 16384 x 16384 képpontos méretre. A Ctrl + c,v,x funkciók a gomb elengedésre működnek az egui rendszer korlátozása miatt.
    *   **Módosítások exportálása:** Lehetőség van a képernyőn látható módosítások (nagyítás/kicsinyítés, forgatás, LUT effektek) alkalmazásával menteni a képet ("Save View") vagy a vágólapra másolni azt ("Copy View").
    *   **Prémium átméretezés:** Mentésnél és másolásnál az alkalmazás Lanczos3 mintavételezést használ, ami tűéles minőséget biztosít kicsinyítés esetén is.
	
![IView preview](screenshots/preview_a.png)

---
### 📖 Használati útmutató

*   **📂 Képkezelés és Böngészés**

    *   **Indítás:** A programot indíthatod parancssorból, vagy az ikonjára kattintva.
    *   **Megnyitás:** Megnyitáskor a parancssorban levő képet, vagy a parancsikonra húzott képet, ennek hiányában a vágólapon levő képet, ennek hiányában a feljövő dialógban megadott képet nyitja meg. Így a böngésződben másolt kép azonnal megnézhető, és átalakítható. A dialógban való megszakítással le is állíthatod a programot, és a korábban használt képekből választhatsz.
    *   **A kép váltása:** Menet közbeni újabb képek megnyitására használd a File/Open menüpontot, vagy húzz be egy képet az ablakba (Drag & Drop), vagy a vágólapról másolj, vagy navigálj a könyvtárban levő képeken előre, vagy hátra a megadott rendezési sorrend szerint.

*   **🎨 Szerkesztés és Megjelenítés**

    *   **Pozíció:** A megjelenített kép vagy a képernyő közepén, vagy a bal felső sarokban jelenik meg. Az ablak elhúzható, de képváltáskor újra pozicionálja az ablakot.
    *   **Nagyítás:** A csúszkával, egérgörgővel, vagy nenüből 0.1x és 10x közötti mérettartományt érhetsz el. Az ablak maximum a képernyő nagyságáig növekszik, a nem látható részeket a kép húzásával, vagy a csúszkával mozgathatjuk az ablakon belül.
    *   **Képkorrekció:** Állítsd a Gammát, Kontrasztot és Világosságot valós időben. A Color menüben ki/be kapcsolhatod a piros, zöld és kék csatornákat, inverz színeket is beállíthatsz. Használhatod a Blur/Sharpen funkciókat a határok elmosására, vagy élesítésére, egy színtartományt átlátszóvá tehetsz.
    *   **Háttérstílusok:** Átlátszó (Png/WebP/Bmp/Tiff) képek esetén a View -> Background Style menüben választhatsz fekete, fehér, szürke vagy a különböző sakktábla minták között.
    *   **Info:** Sok kép tartalmazhat extra információkat (exif data) amelyek megtekinthetők, az esetleges GPS koordináták megnyithatók a Google Map oldalon.

*   **💾 Mentés és Exportálás**

    *   **Save:** Elmenti az eredeti képet, miközben más kép formátumra válthatsz. Jpeg és Webp esetén a mentés képminőségét is beállíthatod.
    *   **Save View:** Elmenti a képet a jelenlegi módosításokkal (forgatás, színek, nagyítás). Ha 0.5x nagyításon állsz, a kép feleakkora méretben kerül mentésre.
    *   **Copy:** Az eredet képet teszi a vágólapra, így más programok közvetlenül átvehetik azt (rgba színmodell).
    *   **Copy View:** A módosított képet teszi a vágólapra, tűéles Lanczos3 újramintavételezéssel.
    *   **Paste:** A vágólapon levő képet behozza a programba.
    *   **Change:** Az eredeti képet a vágólapra teszi, miközben az ott levő képet hozza be programba.
    *   **Change View:** A módosított képet a vágólapra teszi, miközben az ott levő képet hozza be. Ez a módosítások ismétlését teszi lehetővé.
    *   **Formátumok:** Támogatott olvasási/mentési típusok: .jpg, .png, .webp, .tif, .bmp, .gif. Animált képeknél jelenleg az első képet olvassa.
    *   **Korlátozás:** A Ctrl + c,v,x függvények a gomb elengedésekor működnek az egui rendszer korlátai miatt.
    *   **GPS adatok:** Ha a kép tartalmaz geolokációs metaadatokat, az Info panelen megjelenik egy gomb, amellyel a helyszín közvetlenül megnyitható a Google Maps-en.

---

### ⌨️ Shortcuts / Gyorsbillentyűk

| Key | Function |
| --- | --- |
| + / - | Zoom in / out |
| B / N | Before / Next image in directory |
| O | Open image |
| R | Reopen same image (hide/show inside/outside modification)|
| S | Save image  & convert to other type) |
| Shift + S | Save modified view & convert |
| Ctrl + C | Copy to clipboard |
| Ctrl + Shift + C | Copy View to clipboard |
| Ctrl + V | Paste from clipboard |
| Ctrl + X | Change with clipboard |
| Ctrl + Shift + X | Change View with clipboard |
| Escape | exit from popup windows or program  |
| Enter | exit from popup windows or program  |
| Ctrl + R | Toggle red channel |
| Ctrl + G | Toggle greeen channel |
| Ctrl + B | Toggle blue channel |
| Ctrl + I | Invert color channels |
| C | Open color corrections window |
| I | Open informations window |
| G | Toggle backgrounds style for transparent images |
| Ctrl + Left | Rotate -90° |
| Ctrl + Rigth | Rotate 90° |
| Ctrl + Up | Rotate 180° |
| Ctrl + Down | Stand to 0° |
| Ctrl | Pick Pixel to Tooltip (until press). Select color with click. |
| Shift + Alt | Show original image (until press). Warning! You change the keyboard language also. |
| Space | Animation play/stop |
| Left | Animation previous frame |
| Rigth | Animation next frame |

---
### 🛠 Tech Stack / Technológiai háttér

*   **Language:** [Rust](https://www.rust-lang.org)
*   **UI Framework:** [eframe] / [egui] 

*   **Cross-platform:** Tested and working on Windows 10 and Linux (Linux Mint).

*   **Executables:** in the executables folder
*   **Latest Version:** 0.8.1 

### 🚀 Development / Fejlesztés

```bash
# Build and run the project
# Projekt fordítása és futtatása
git clone https://github.com/Ferenc-Takacs/IView.git
cd IView
cargo run --release
```
![IView preview](screenshots/preview.jpg)