@echo off
setlocal enabledelayedexpansion

REM ==============================================================================
REM == ��������� ������������������ ������ ��� ������ � ������ ==
REM ==============================================================================

REM --- ��� 0: ����訢��� ᮮ�饭�� ��� ������ ---
set "COMMIT_MSG="
set /p COMMIT_MSG="������ ᮮ�饭�� ��� ������ (����뢠�饥 ��������� � ��室��� ����): 

REM --- ��� 1: ������ � ��� ��������� � 'main' ---
echo ��� 1: ������ � ��� ��������� � 'main'...
git checkout main
if %errorlevel% neq 0 (
    echo ? �訡��: �� 㤠���� ��४������� �� ���� 'main'.
    goto :eof
)
git add .

REM ���������� !COMMIT_MSG! ������ %COMMIT_MSG% ��� ���������� ��������� ������������
git commit -m "!COMMIT_MSG!"
if %errorlevel% neq 0 (
    echo ??  �।�०�����: ��������, ��祣� ��������. ��ਯ� �த����� �믮������.
)

git push origin main
if %errorlevel% neq 0 (
    echo ? �訡��: �� 㤠���� ������� ��������� � 'main'.
    goto :eof
)
echo ? ��������� � 'main' �ᯥ譮 ��ࠢ����.
for /f "delims=" %%a in ('git rev-parse --short HEAD') do set "LAST_COMMIT_HASH=%%a"

REM --- ��� 2: ���ઠ Wasm ---
echo.
echo >>> ��� 2: ���ઠ Wasm-�����...
wasm-pack build --target web
if %errorlevel% neq 0 (
    echo ? �訡��: ���ઠ Wasm �� 㤠����.
    goto :eof
)
echo ? ���ઠ Wasm �����襭�.

REM --- ��� 3: ������ � 'gh-pages' ---
echo.
echo >>> ��� 3: ������ ���䠪⮢ � 'gh-pages'...
git checkout gh-pages
if %errorlevel% neq 0 (
    echo ? �訡��: �� 㤠���� ��४������� �� 'gh-pages'.
    git checkout main
    goto :eof
)

echo     -> ��६�饭�� ���䠪⮢ ᡮન � ���⪠...
if not exist "pkg\pathfinder_bg.wasm" (
    echo ? �訡��: �� ������� 䠩�� ᡮન � ��४�ਨ 'pkg'.
    git checkout main
    goto :eof
)

move pkg\pathfinder_bg.wasm .
move pkg\pathfinder.js .
rmdir /s /q pkg

echo     -> ������ � ��� � 'gh-pages'...
git add pathfinder_bg.wasm pathfinder.js
git commit -m "deploy: ���ઠ �� ������ main@%LAST_COMMIT_HASH%"
git push origin gh-pages
if %errorlevel% neq 0 (
    echo ? �訡��: �� 㤠���� ������� ��������� � 'gh-pages'.
    git checkout main
    goto :eof
)
echo ? ���䠪�� �ᯥ譮 ����������.

REM --- ��� 4: �����饭�� �� 'main' ---
echo.
echo >>> ��� 4: �����饭�� �� ���� 'main'...
git checkout main

echo.
echo ?? ��� ��⮢�! ��室�� ��� � ᡮઠ �뫨 �ᯥ譮 ��ࠢ���� �� GitHub.
pause