@echo off
set /p "min=请输入多少分钟后关机，或输入0取消关机："
echo %min%|findstr /r "^[0-9]*$">nul
if errorlevel 1 (
    echo 输入错误
    pause
    exit
)


if %min%==0 (
    shutdown -a
    echo 取消关机
    pause
    exit
)

set /a time=%min%*60
shutdown -a
shutdown -s -t %time%

