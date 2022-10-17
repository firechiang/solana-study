##### 代码格式化基础
```shell
# 安装代码格式化插件（注意：代码格式化规则配置在 .prettierrc.json 文件里面）
$ yarn add --dev --exact prettier

# 手动格式化代码（. 表示格式化全部文件）
$ yarn prettier --write .
```

##### 代码在提交之前自动被格式化
```shell
# 安装插件
$ yarn add --dev husky lint-staged
```