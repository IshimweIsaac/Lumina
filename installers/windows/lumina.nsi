; Lumina v2.0.0 Modern Installer Script
!include "MUI2.nsh"
!include "EnvVarUpdate.nsh"

; --- Product Info ---
!define APPNAME "Lumina"
!define COMPANYNAME "LuminaLang"
!define DESCRIPTION "Declarative reactive language for infrastructure monitoring"
!define VERSION "2.0.0"
!define VERSIONMAJOR 2
!define VERSIONMINOR 0
!define VERSIONBUILD 0

; --- Build target selection ---
; Pass -DTARGET=x86_64-pc-windows-msvc on the makensis command line to override.
; Defaults to x86_64-pc-windows-gnu for local cross-compilation from Linux.
!ifndef TARGET
  !define TARGET "x86_64-pc-windows-gnu"
!endif

; --- Configuration ---
Name "${APPNAME} ${VERSION}"
OutFile "Lumina-v${VERSION}-x64-Setup.exe"
InstallDir "$PROGRAMFILES64\${COMPANYNAME}\${APPNAME}"
InstallDirRegKey HKLM "Software\${COMPANYNAME}\${APPNAME}" "Install_Dir"
RequestExecutionLevel admin

; --- UI Settings ---
!define MUI_ABORTWARNING
!define MUI_ICON "assets\logo.ico"
!define MUI_UNICON "assets\logo.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "assets\header.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "assets\welcome.bmp"

; --- Pages ---
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; --- Language ---
!insertmacro MUI_LANGUAGE "English"

; --- Install Section ---
Section "Lumina Runtime" SecRuntime
  SetOutPath "$INSTDIR"
  
  ; Binaries — uses the TARGET define so both local GNU and CI MSVC builds work
  File "/oname=lumina.exe" "..\..\target\${TARGET}\release\lumina.exe"
  File "/oname=lumina-lsp.exe" "..\..\target\${TARGET}\release\lumina-lsp.exe"
  File "..\..\extensions\lumina-vscode\lumina-lang-1.8.0.vsix"
  
  ; Write installation path to registry
  WriteRegStr HKLM "Software\${COMPANYNAME}\${APPNAME}" "Install_Dir" "$INSTDIR"
  
  ; Add to PATH (System wide)
  ${EnvVarUpdate} $0 "PATH" "A" "HKLM" "$INSTDIR"

  ; --- File Associations (.lum) ---
  WriteRegStr HKCR ".lum" "" "LuminaSourceFile"
  WriteRegStr HKCR "LuminaSourceFile" "" "Lumina Source File"
  WriteRegStr HKCR "LuminaSourceFile\DefaultIcon" "" "$INSTDIR\lumina.exe,0"
  WriteRegStr HKCR "LuminaSourceFile\shell\open\command" "" '"$INSTDIR\lumina.exe" run "%1"'
  
  ; Notify Shell of changes
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'

  ; Uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
  ; Add/Remove Programs metadata
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "Lumina Programming Language"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${COMPANYNAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "URLInfoAbout" "https://lumina-lang.dev"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\lumina.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  
  DetailPrint "Initializing Lumina ecosystem and IDE extensions..."
  ExecWait '"$INSTDIR\lumina.exe" setup'
SectionEnd

; --- Uninstall Section ---
Section "Uninstall"
  ; Remove from PATH
  ${EnvVarUpdate} $0 "PATH" "R" "HKLM" "$INSTDIR"

  ; Clean files
  Delete "$INSTDIR\lumina.exe"
  Delete "$INSTDIR\lumina-lsp.exe"
  Delete "$INSTDIR\lumina-lang-1.8.0.vsix"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  RMDir "$PROGRAMFILES64\${COMPANYNAME}"

  ; Clean registry
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
  DeleteRegKey HKLM "Software\${COMPANYNAME}\${APPNAME}"
  
  ; Remove File Associations
  DeleteRegKey HKCR ".lum"
  DeleteRegKey HKCR "LuminaSourceFile"
  
  ; Notify Shell of changes
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd
