# db_export

[![CI](https://github.com/iouts/db_export/actions/workflows/build.yml/badge.svg)](https://github.com/iouts/db_export/actions/workflows/build.yml)

**注意：由于时光印记作者重新进行了压缩定制打包，本工具已无法正确解压。**

用于获得[时光印记]压缩包里面的图片。

[时光印记]中的压缩包包含两个文件exe和db。其中exe文件含有`Trojan:Win32/Glupteba!ml`木马**病毒**，`db`文件为SQLite数据库。

本程序用于导出其中的JPEG图片。语法为：

`db-export <db文件名> <输出目录名称>`

[时光印记]:http://d.sundx.cn/
