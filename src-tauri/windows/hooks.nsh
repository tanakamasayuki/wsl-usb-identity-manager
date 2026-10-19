; Installer hooks, included by the NSIS template Tauri generates.
;
; The template closes a running copy before replacing its files and starts it
; again only when given `/R`, which winget never passes. An upgrade therefore
; left the application closed until the next sign-in (R12.10). These hooks note
; whether it was running before the template closes it, and start it again
; afterwards — in the tray, the same way the `Run` entry starts it.

Var WuimWasRunning

!macro NSIS_HOOK_PREINSTALL
  ; Runs before the template's CheckIfAppIsRunning, so the copy is still alive.
  StrCpy $WuimWasRunning 0
  nsis_tauri_utils::FindProcessCurrentUser "${MAINBINARYNAME}.exe"
  Pop $R0
  ${If} $R0 = 0
    StrCpy $WuimWasRunning 1
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $WuimWasRunning = 1
    nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" "--hidden"
  ${EndIf}
!macroend
