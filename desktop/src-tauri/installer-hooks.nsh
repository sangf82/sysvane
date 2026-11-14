; Hooks for the Tauri NSIS installer (per machine, into Program Files, run with administrator rights).
; PawnIO is pinned to one release and checked against its SHA-256 before it runs.

!define PAWNIO_URL "https://github.com/namazso/PawnIO.Setup/releases/download/2.2.0/PawnIO_setup.exe"
!define PAWNIO_SHA256 "1F519A22E47187F70A1379A48CA604981C4FCF694F4E65B734AAA74A9FBA3032"
!define PAWNIO_DIR "$PROGRAMFILES64\PawnIO"

!macro NSIS_HOOK_PREINSTALL
  ; The core and the window hold their exe files open, so stop them before files are replaced.
  nsExec::Exec 'taskkill /F /IM sysvane-ui.exe'
  nsExec::Exec 'taskkill /F /IM sysvane-core.exe'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Install PawnIO only when it is missing. An existing copy is left alone and never removed by us.
  IfFileExists "${PAWNIO_DIR}\PawnIOLib.dll" pawnio_done
    DetailPrint "Installing PawnIO (needed for CPU temperature)"
    nsExec::ExecToLog `powershell -NoProfile -ExecutionPolicy Bypass -Command "$$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri '${PAWNIO_URL}' -OutFile '$TEMP\PawnIO_setup.exe'"`
    Pop $0
    StrCmp $0 "0" 0 pawnio_failed
    nsExec::ExecToLog `powershell -NoProfile -ExecutionPolicy Bypass -Command "if ((Get-FileHash '$TEMP\PawnIO_setup.exe' -Algorithm SHA256).Hash -ne '${PAWNIO_SHA256}') { exit 2 }"`
    Pop $0
    StrCmp $0 "0" 0 pawnio_failed
    nsExec::ExecToLog '"$TEMP\PawnIO_setup.exe" -install -silent'
    Pop $0
    IfFileExists "${PAWNIO_DIR}\PawnIOLib.dll" 0 pawnio_failed
      WriteRegDWORD HKLM "Software\SysVane" "InstalledPawnIO" 1
      Goto pawnio_cleanup
  pawnio_failed:
    DetailPrint "PawnIO was not installed. CPU temperature stays off until it is."
  pawnio_cleanup:
    Delete "$TEMP\PawnIO_setup.exe"
  pawnio_done:
  ; The scheduled task starts the core with administrator rights. "Start with Windows" stays off.
  nsExec::ExecToLog '"$INSTDIR\sysvane-core.exe" --register-task'
  Pop $0
  ; An update (the UI passes /UPDATE) brings the core back; a fresh install leaves it to the user.
  ${If} $UpdateMode = 1
    nsExec::Exec 'schtasks /run /tn "SysVane Core"'
    Pop $0
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'taskkill /F /IM sysvane-ui.exe'
  nsExec::Exec 'taskkill /F /IM sysvane-core.exe'
  nsExec::ExecToLog '"$INSTDIR\sysvane-core.exe" --unregister-task'
  Pop $0
  ; Remove PawnIO only if this installer put it there.
  ReadRegDWORD $1 HKLM "Software\SysVane" "InstalledPawnIO"
  StrCmp $1 "1" 0 +3
    IfFileExists "${PAWNIO_DIR}\uninstall.exe" 0 +2
      nsExec::ExecToLog '"${PAWNIO_DIR}\uninstall.exe" -uninstall -silent'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKLM "Software\SysVane"
  MessageBox MB_YESNO|MB_ICONQUESTION "Also delete SysVane settings and logs?" /SD IDNO IDNO +3
    RMDir /r "$%ProgramData%\SysVane"
    RMDir /r "$LOCALAPPDATA\com.sysvane.app"
!macroend
