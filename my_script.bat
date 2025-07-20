@echo off
setlocal enabledelayedexpansion

REM =============================================
REM == BUILD AND DEPLOY SCRIPT ==
REM =============================================

REM Step 0: Prompt for commit message
set "COMMIT_MSG="
set /p COMMIT_MSG="Enter commit message: "
if not defined COMMIT_MSG (
    echo Error: Commit message cannot be empty.
    goto :eof
)

REM Step 1: Commit and push changes to 'main'
echo.
echo Step 1: Committing and pushing changes to 'main'...
git checkout main
if %errorlevel% neq 0 (
    echo Error: Failed to switch to 'main' branch.
    goto :eof
)
git add .

REM Using !COMMIT_MSG! for safe handling of special characters
git commit -m "!COMMIT_MSG!"
if %errorlevel% neq 0 (
    echo Warning: Nothing to commit, maybe. Script will continue.
)

git push origin main
if %errorlevel% neq 0 (
    echo Error: Failed to push changes to 'main'.
    goto :eof
)
echo OK: Changes pushed to 'main' successfully.
for /f "delims=" %%a in ('git rev-parse --short HEAD') do set "LAST_COMMIT_HASH=%%a"

REM Step 2: Build Wasm
echo.
echo Step 2: Building Wasm module...
wasm-pack build --target web
if %errorlevel% neq 0 (
    echo Error: Wasm build failed.
    goto :eof
)
echo OK: Wasm build finished.

REM Step 3: Deploy to 'gh-pages'
echo.
echo Step 3: Deploying artifacts to 'gh-pages'...
git checkout gh-pages
if %errorlevel% neq 0 (
    echo Error: Failed to switch to 'gh-pages'.
    git checkout main
    goto :eof
)

REM =================================================================
REM == КЛЮЧЕВОЕ ИЗМЕНЕНИЕ: Синхронизация с удаленным репозиторием ==
REM =================================================================
echo Synchronizing with remote 'gh-pages' branch...
git fetch origin
git reset --hard origin/gh-pages
if %errorlevel% neq 0 (
    echo Error: Failed to reset local branch to remote state.
    git checkout main
    goto :eof
)

echo Moving build artifacts and cleaning up...
if not exist "pkg\pathfinder_bg.wasm" (
    echo Error: Build files not found in 'pkg' directory.
    git checkout main
    goto :eof
)

move pkg\pathfinder_bg.wasm .
move pkg\pathfinder.js .
rmdir /s /q pkg

echo Committing and pushing to 'gh-pages'...
git add .
git commit -m "deploy: Build from commit main@%LAST_COMMIT_HASH%"

REM =================================================================
REM == КЛЮЧЕВОЕ ИЗМЕНЕНИЕ: Теперь можно использовать обычный push ==
REM =================================================================
git push origin gh-pages
if %errorlevel% neq 0 (
    echo Error: Failed to push changes to 'gh-pages'.
    git checkout main
    goto :eof
)
echo OK: Artifacts deployed successfully.

REM Step 4: Return to 'main'
echo.
echo Step 4: Returning to 'main' branch...
git checkout main

echo.
echo All done!
pause