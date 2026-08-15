!include "LogicLib.nsh"

!macro CopyWinDivertFiles
  DetailPrint "Installing WinDivert kernel driver and libraries..."
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert.dll" "$INSTDIR\WinDivert.dll"
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert64.sys" "$INSTDIR\WinDivert64.sys"
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert64.sys" "$INSTDIR\WinDivert.sys"
  CopyFiles /SILENT "$INSTDIR\bin\WinDivert.dll" "$INSTDIR\WinDivert.dll"
  CopyFiles /SILENT "$INSTDIR\bin\WinDivert64.sys" "$INSTDIR\WinDivert64.sys"
  CopyFiles /SILENT "$INSTDIR\bin\WinDivert64.sys" "$INSTDIR\WinDivert.sys"
  CopyFiles /SILENT "$INSTDIR\resources\WinDivert.dll" "$INSTDIR\WinDivert.dll"
  CopyFiles /SILENT "$INSTDIR\resources\WinDivert64.sys" "$INSTDIR\WinDivert64.sys"
  CopyFiles /SILENT "$INSTDIR\resources\WinDivert64.sys" "$INSTDIR\WinDivert.sys"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro CopyWinDivertFiles
!macroend

!macro customInstall
  !insertmacro CopyWinDivertFiles
!macroend
