@echo off
set /p "min=��������ٷ��Ӻ�ػ���������0ȡ���ػ���"
echo %min%|findstr /r "^[0-9]*$">nul
if errorlevel 1 (
    echo �������
    pause
    exit
)


if %min%==0 (
    shutdown -a
    echo ȡ���ػ�
    pause
    exit
)

set /a time=%min%*60
shutdown -a
shutdown -s -t %time%

