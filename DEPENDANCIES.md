# ✅ Installation de OpenCV pour Rust sous Windows (via vcpkg)

## 1. 📥 Install OpenCV & LLVM

```bash
choco install llvm opencv
```

## 📦 2. Cloner et configurer vcpkg

```bash
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
```

or

```bash
choco install llvm opencv
choco install pkgconfiglite
```

## 📥 3. Installer OpenCV avec les modules souhaités

```bash
vcpkg install opencv4[core,imgproc,highgui,contrib] --triplet x64-windows
```

## 📁 4. Configurer les variables d’environnement (dans CMD)

```bash
set TESSERACT_PATH="C:\Program Files\Tesseract-OCR\tesseract.exe"
set OPENCV_INCLUDE_PATHS="C:\tools\opencv\build\include"
set OPENCV_LINK_PATHS="C:\tools\opencv\build\x64\vc15\lib"
set OPENCV_LINK_LIBS="opencv_world480"
set PKG_CONFIG_PATH="C:\vcpkg\installed\x64-windows\lib\pkgconfig"
set OPENCV_ROOT="C:\vcpkg\installed\x64-windows\share\opencv4"
set PATH="%PATH%;C:\tools\opencv\build\bin"  # Add OpenCV bin directory to PATH
```
