# !/usr/bin/env python3
# -*- coding: utf-8 -*-

'''
@Time    : 2023/12/16
@Author  : Ruan Jiazhen
@Description: 将zip压缩包内所有webp图片转换为jpg格式图片

安装依赖库
pip install send2trash
pip install pillow
'''

import os
import sys
import zipfile
from PIL import Image
from send2trash import send2trash

# 打印当前运行目录，让用户确认或直接输入目标目录，返回该目录下所有压缩包的绝对路径


def getZipPath():
    print('当前运行目录：', os.getcwd())
    targetPath = input('请输入目标目录或直接回车确认：')
    if targetPath == '':
        targetPath = os.getcwd()
    if os.path.exists(targetPath):
        zipPathList = []
        for root, dirs, files in os.walk(targetPath):
            for file in files:
                if file.endswith('.zip'):
                    zipPathList.append(os.path.join(root, file))
        return zipPathList
    else:
        print('目标目录不存在！')
        sys.exit()

# 检测目标压缩包内是否包含webp格式的图片


def checkWebp(zipPath):
    zipFile = zipfile.ZipFile(zipPath)
    for file in zipFile.namelist():
        if file.endswith('.webp'):
            return True
    return False

# 将包含有webp格式图片的压缩包内的webp图片转换为jpg格式，并删除原webp图片，重新压缩该压缩包


def convertWebp(zipPath):
    targetZipFile = zipfile.ZipFile(zipPath, 'a')
    fileList = targetZipFile.namelist()
    # 如果所有文件均不是webp格式，则不处理，直接返回，否则解压该压缩包
    webpFlag = False
    for file in fileList:
        if file.endswith('.webp'):
            webpFlag = True
            break

    if not webpFlag:
        return

    # 解压所有文件到该压缩文件对应的临时目录
    targetZipFileName = os.path.basename(zipPath)
    tempdir = os.path.join(os.getcwd(), targetZipFileName.replace('.zip', ''))
    # 创建临时目录
    if not os.path.exists(tempdir):
        os.mkdir(tempdir)
    targetZipFile.extractall(tempdir)
    # 转换webp图片为jpg格式
    for root, dirs, files in os.walk(tempdir):
        for file in files:
            if file.endswith('.webp'):
                webpPath = os.path.join(root, file)
                im = Image.open(webpPath).convert('RGB')
                im.save(webpPath.replace('.webp', '.jpg'), 'jpeg')
                # 删除原webp图片
                os.remove(webpPath)

    targetZipFile.close()
    send2trash(zipPath)

    # 重新压缩该压缩包
    newZipFile = zipfile.ZipFile(zipPath, 'w')
    for root, dirs, files in os.walk(tempdir):
        for file in files:
            newZipFile.write(os.path.join(root, file), file)

    # 删除临时目录下所有文件，然后删除临时目录
    for root, dirs, files in os.walk(tempdir):
        for file in files:
            os.remove(os.path.join(root, file))
    os.rmdir(tempdir)


# 主函数
def main():
    zipPathList = getZipPath()
    print('目标压缩包：', zipPathList)
    for zipPath in zipPathList:
        print('正在处理：', zipPath)
        if checkWebp(zipPath):
            convertWebp(zipPath)
            print('已处理：', zipPath)
        else:
            print('无需处理：', zipPath)


if __name__ == '__main__':
    main()
