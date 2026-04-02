!define APPNAME "Lumina"
!define COMPANYNAME "LuminaLang"
!define DESCRIPTION "Declarative reactive language for IoT and infrastructure monitoring"
!define VERSIONMAJOR 1
!define VERSIONMINOR 7
!define VERSIONBUILD 0

Name "${APPNAME} ${VERSIONMAJOR}.${VERSIONMINOR}.${VERSIONBUILD}"
OutFile "LuminaSetup.exe"
InstallDir "$PROGRAMFILES\${COMPANYNAME}\${APPNAME}"

; Request application privileges for Windows Vista+
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Install"
  ; Set output path to the installation directory.
  SetOutPath "$INSTDIR"

  ; Put files there
  File "..\..\target\x86_64-pc-windows-msvc\release\lumina.exe"
  File "..\..\target\x86_64-pc-windows-msvc\release\lumina-lsp.exe"

  ; Add bin folder to user PATH
  EnVar::SetHKCU
  EnVar::AddValue "Path" "$INSTDIR"

  ; Write uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\lumina.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${COMPANYNAME}"
SectionEnd

Section "Uninstall"
  ; Remove from PATH
  EnVar::SetHKCU
  EnVar::DeleteValue "Path" "$INSTDIR"

  ; Remove registries
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"

  ; Remove files
  Delete "$INSTDIR\lumina.exe"
  Delete "$INSTDIR\lumina-lsp.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
