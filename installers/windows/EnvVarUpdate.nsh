/**
 * EnvVarUpdate
 * Function: Adds or removes a string from an environment variable
 * (like PATH or LIB)
 * 
 * Usage:
 *   ${EnvVarUpdate} "ResultVar" "VarName" "Action" "Scope" "String"
 * 
 * ResultVar: Variable to store the result (usually $0)
 * VarName: Name of the environment variable (e.g. PATH)
 * Action: "A" for Append, "P" for Prepend, "R" for Remove
 * Scope: "HKLM" for All Users, "HKCU" for Current User
 * String: String to add or remove
 *
 * This version correctly provides un.* function variants for use
 * in uninstall sections.
 */

!ifndef ENVVARUPDATE_NSH
!define ENVVARUPDATE_NSH

!include "WinMessages.nsh"

; --- Installer macro ---
!define EnvVarUpdate `!insertmacro EnvVarUpdate`

!macro EnvVarUpdate ResultVar VarName Action Scope String
  Push "${VarName}"
  Push "${Action}"
  Push "${Scope}"
  Push "${String}"
  !ifdef __UNINSTALL__
    Call un.EnvVarUpdate
  !else
    Call EnvVarUpdate
  !endif
  Pop "${ResultVar}"
!macroend

; =====================================================================
; We use a macro-based approach to generate both the installer and
; uninstaller variants of every function from a single definition.
; =====================================================================

; --- StrStr ---
!macro _StrStr_Func UN
Function ${UN}StrStr
  Exch $R1 ; string to find
  Exch
  Exch $R0 ; string to search in
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4

  StrLen $2 $R1
  StrLen $1 $R0
  IntOp $4 $1 - $2
  StrCpy $3 0

${UN}StrStr_Loop:
  StrCpy $0 $R0 $2 $3
  StrCmp $0 $R1 ${UN}StrStr_Found
  IntCmp $3 $4 ${UN}StrStr_Done ${UN}StrStr_Done
  IntOp $3 $3 + 1
  Goto ${UN}StrStr_Loop

${UN}StrStr_Found:
  StrCpy $R0 $R0 "" $3
  Goto ${UN}StrStr_End

${UN}StrStr_Done:
  StrCpy $R0 ""

${UN}StrStr_End:
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
  Pop $R1
  Exch $R0
FunctionEnd
!macroend

; --- StrReplace ---
!macro _StrReplace_Func UN
Function ${UN}StrReplace
  Exch $R2 ; replace with
  Exch
  Exch $R1 ; find
  Exch 2
  Exch $R0 ; in
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5

  StrLen $1 $R1
  StrCpy $5 ""

${UN}StrReplace_Loop:
  Push $R0
  Push $R1
  Call ${UN}StrStr
  Pop $0
  StrCmp $0 "" ${UN}StrReplace_Done

  StrLen $2 $R0
  StrLen $3 $0
  IntOp $4 $2 - $3
  StrCpy $2 $R0 $4
  StrCpy $5 "$5$2$R2"
  StrCpy $R0 $0 "" $1
  Goto ${UN}StrReplace_Loop

${UN}StrReplace_Done:
  StrCpy $R0 "$5$R0"
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
  Pop $R2
  Pop $R1
  Exch $R0
FunctionEnd
!macroend

; --- RemoveFromPath ---
!macro _RemoveFromPath_Func UN
Function ${UN}RemoveFromPath
  Exch $2 ; String to remove
  Exch
  Exch $1 ; Current Path
  Push $3
  Push $4
  Push $5

  StrCpy $3 $1
  StrLen $4 $2

${UN}RemoveFromPath_Loop:
  Push $3
  Push $2
  Call ${UN}StrStr
  Pop $5
  StrCmp $5 "" ${UN}RemoveFromPath_Done

  ; Found the string — remove it
  StrLen $0 $3
  StrLen $R0 $5
  IntOp $0 $0 - $R0 ; offset

  StrCpy $R1 $3 $0 ; part before
  IntOp $0 $0 + $4
  StrCpy $3 $3 "" $0 ; part after

  StrCpy $3 "$R1$3"
  Goto ${UN}RemoveFromPath_Loop

${UN}RemoveFromPath_Done:
  ; Clean up double semicolons
  Push $3
  Push ";;"
  Push ";"
  Call ${UN}StrReplace
  Pop $3

  ; Trim leading semicolons
  StrCpy $0 $3 1
  StrCmp $0 ";" 0 +2
  StrCpy $3 $3 "" 1

  ; Trim trailing semicolons
  StrLen $0 $3
  IntOp $0 $0 - 1
  StrCpy $R0 $3 1 $0
  StrCmp $R0 ";" 0 +2
  StrCpy $3 $3 $0

  Pop $5
  Pop $4
  Pop $3
  Pop $1
  Exch $3
  Pop $2
  Push $3
FunctionEnd
!macroend

; --- EnvVarUpdate ---
!macro _EnvVarUpdate_Func UN
Function ${UN}EnvVarUpdate
  Push $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  Push $R0
  Push $R1
  Push $R2
  Push $R3
  Push $R4
  Push $R5

  Pop $R5 ; String
  Pop $R4 ; Scope
  Pop $R3 ; Action
  Pop $R2 ; VarName

  ; Check Action
  StrCmp $R3 "A" ${UN}EVU_ActionOk
  StrCmp $R3 "P" ${UN}EVU_ActionOk
  StrCmp $R3 "R" ${UN}EVU_ActionOk
  Abort "EnvVarUpdate: Invalid Action ($R3)"

${UN}EVU_ActionOk:
  ; Check Scope
  StrCmp $R4 "HKLM" ${UN}EVU_ScopeOk
  StrCmp $R4 "HKCU" ${UN}EVU_ScopeOk
  Abort "EnvVarUpdate: Invalid Scope ($R4)"

${UN}EVU_ScopeOk:
  ; Read the current value
  StrCpy $R0 ""
  StrCmp $R4 "HKLM" ${UN}EVU_ReadHKLM
  ReadRegStr $R0 HKCU "Environment" "$R2"
  Goto ${UN}EVU_ReadDone
${UN}EVU_ReadHKLM:
  ReadRegStr $R0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "$R2"
${UN}EVU_ReadDone:

  ; If removing, do it now
  StrCpy $1 $R0
  StrCpy $2 $R5
  Call ${UN}RemoveFromPath
  Pop $R1 ; Cleaned value

  StrCmp $R3 "R" ${UN}EVU_WriteBack

  ; Adding: Append or Prepend
  StrCmp $R1 "" ${UN}EVU_AddEmpty
  StrCmp $R3 "A" ${UN}EVU_AddAppend
  StrCpy $R1 "$R5;$R1"
  Goto ${UN}EVU_WriteBack
${UN}EVU_AddAppend:
  StrCpy $R1 "$R1;$R5"
  Goto ${UN}EVU_WriteBack
${UN}EVU_AddEmpty:
  StrCpy $R1 "$R5"

${UN}EVU_WriteBack:
  ; Write the new value
  StrCmp $R4 "HKLM" ${UN}EVU_WriteHKLM
  WriteRegExpandStr HKCU "Environment" "$R2" "$R1"
  Goto ${UN}EVU_WriteDone
${UN}EVU_WriteHKLM:
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "$R2" "$R1"
${UN}EVU_WriteDone:

  ; Broadcast change
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
  Push "OK"
FunctionEnd
!macroend

; =====================================================================
; Instantiate both installer and uninstaller versions
; =====================================================================
!insertmacro _StrStr_Func ""
!insertmacro _StrStr_Func "un."

!insertmacro _StrReplace_Func ""
!insertmacro _StrReplace_Func "un."

!insertmacro _RemoveFromPath_Func ""
!insertmacro _RemoveFromPath_Func "un."

!insertmacro _EnvVarUpdate_Func ""
!insertmacro _EnvVarUpdate_Func "un."

!endif
