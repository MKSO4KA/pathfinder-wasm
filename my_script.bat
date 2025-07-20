@echo off
setlocal enabledelayedexpansion

REM ==============================================================================
REM == ПОЛНОСТЬЮ АВТОМАТИЗИРОВАННЫЙ СКРИПТ ДЛЯ СБОРКИ И ДЕПЛОЯ ==
REM ==============================================================================

REM --- Шаг 0: Запрашиваем сообщение для коммита ---
set "COMMIT_MSG="
set /p COMMIT_MSG="Введите сообщение для коммита (описывающее изменения в исходном коде): 

REM --- Шаг 1: Коммит и пуш изменений в 'main' ---
echo Шаг 1: Коммит и пуш изменений в 'main'...
git checkout main
if %errorlevel% neq 0 (
    echo ? Ошибка: Не удалось переключиться на ветку 'main'.
    goto :eof
)
git add .

REM ИСПОЛЬЗУЕМ !COMMIT_MSG! ВМЕСТО %COMMIT_MSG% ДЛЯ БЕЗОПАСНОЙ ОБРАБОТКИ СПЕЦСИМВОЛОВ
git commit -m "!COMMIT_MSG!"
if %errorlevel% neq 0 (
    echo ??  Предупреждение: Возможно, нечего коммитить. Скрипт продолжит выполнение.
)

git push origin main
if %errorlevel% neq 0 (
    echo ? Ошибка: Не удалось запушить изменения в 'main'.
    goto :eof
)
echo ? Изменения в 'main' успешно отправлены.
for /f "delims=" %%a in ('git rev-parse --short HEAD') do set "LAST_COMMIT_HASH=%%a"

REM --- Шаг 2: Сборка Wasm ---
echo.
echo >>> Шаг 2: Сборка Wasm-модуля...
wasm-pack build --target web
if %errorlevel% neq 0 (
    echo ? Ошибка: Сборка Wasm не удалась.
    goto :eof
)
echo ? Сборка Wasm завершена.

REM --- Шаг 3: Деплой в 'gh-pages' ---
echo.
echo >>> Шаг 3: Деплой артефактов в 'gh-pages'...
git checkout gh-pages
if %errorlevel% neq 0 (
    echo ? Ошибка: Не удалось переключиться на 'gh-pages'.
    git checkout main
    goto :eof
)

echo     -> Перемещение артефактов сборки и очистка...
if not exist "pkg\pathfinder_bg.wasm" (
    echo ? Ошибка: Не найдены файлы сборки в директории 'pkg'.
    git checkout main
    goto :eof
)

move pkg\pathfinder_bg.wasm .
move pkg\pathfinder.js .
rmdir /s /q pkg

echo     -> Коммит и пуш в 'gh-pages'...
git add pathfinder_bg.wasm pathfinder.js
git commit -m "deploy: Сборка из коммита main@%LAST_COMMIT_HASH%"
git push origin gh-pages
if %errorlevel% neq 0 (
    echo ? Ошибка: Не удалось запушить изменения в 'gh-pages'.
    git checkout main
    goto :eof
)
echo ? Артефакты успешно задеплоены.

REM --- Шаг 4: Возвращение на 'main' ---
echo.
echo >>> Шаг 4: Возвращение на ветку 'main'...
git checkout main

echo.
echo ?? Всё готово! Исходный код и сборка были успешно отправлены на GitHub.
pause