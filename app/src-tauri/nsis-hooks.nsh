!macro customInstall
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert.dll" "$INSTDIR\WinDivert.dll"
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert64.sys" "$INSTDIR\WinDivert64.sys"
  CopyFiles /SILENT "$INSTDIR\resources\bin\WinDivert64.sys" "$INSTDIR\WinDivert.sys"
!macroend
