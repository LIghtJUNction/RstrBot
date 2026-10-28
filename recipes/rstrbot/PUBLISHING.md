# 发布与发现

项目、仓库和产品名保持RstrBot，原远端LIghtJUNction/RstrBot。发布配方的GitHub仓库须含recipkg topic并保留原标签。topic只用于候选发现，不代替配方校验、固定版本或软件验收。

发布前运行八包check、递归pack、归档check并生成完整build提示词。归档只含UTF-8 Markdown和完整LOCK，源实现/缓存/数据库不入包。源码是generated/临时生成物，配方提交不应暗含新实现已通过原生或外部服务验收。正式软件发行须独立验证构建/签名/安装/升级及数据保留。
