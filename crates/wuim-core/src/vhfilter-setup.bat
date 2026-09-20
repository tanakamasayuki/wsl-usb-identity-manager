@echo off
setlocal
title Get vhfilter for WSL USB Identity Manager

rem  Fetches VirtualHere's vhfilter.exe into this folder and installs its filter
rem  driver, which is what per-port power switching needs.
rem
rem  Written here by WSL USB Identity Manager. Edit it freely; it is rewritten
rem  whenever the application is asked for it again.
rem
rem  English only, deliberately: a .bat is read in the console code page, and
rem  anything else turns into mojibake on a machine set up differently.

set "TARGET=%~dp0vhfilter.exe"
set "URL=https://www.virtualhere.com/sites/default/files/usbserver/vhfilterexe/vhfilter.exe"

rem --- --install-filter needs administrator rights; downloading does not ------
net session >nul 2>&1
if not errorlevel 1 goto elevated
echo Asking for administrator rights...
powershell -NoProfile -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
exit /b 0

:elevated
echo.
echo Target : %TARGET%
echo Source : %URL%
echo.

if exist "%TARGET%" (
  echo Removing the previous copy...
  del /f /q "%TARGET%"
  if exist "%TARGET%" (
    echo Could not delete it. Close anything using it and run this again.
    goto fail
  )
)

echo Downloading...
curl -L --fail --show-error -o "%TARGET%" "%URL%"
if errorlevel 1 goto fail
if not exist "%TARGET%" goto fail

rem --- the file is signed, so it is worth checking before running it ---------
echo Checking the signature...
powershell -NoProfile -Command "$s = Get-AuthenticodeSignature -LiteralPath $env:TARGET; if ($s.Status -ne 'Valid') { Write-Host ('  status: ' + $s.Status); exit 1 }; if ($s.SignerCertificate.Subject -notlike '*VirtualHere*') { Write-Host ('  signer: ' + $s.SignerCertificate.Subject); exit 1 }; Write-Host ('  signed by ' + $s.SignerCertificate.Subject.Split(',')[0])"
if errorlevel 1 (
  echo.
  echo Not validly signed by VirtualHere. Deleting the download rather than running it.
  del /f /q "%TARGET%"
  goto fail
)

echo.
echo Installing the filter driver...
"%TARGET%" --install-filter
echo.
echo Done. Windows needs restarting before per-port power switching works.
echo Afterwards this folder is one of the places the application looks, so
echo there is nothing else to set.
echo.
pause
exit /b 0

:fail
echo.
echo Failed. Nothing was installed.
echo.
pause
exit /b 1
