create table if not exists `uv_lamp_mqtt_messages`(
    `id` bigint unsigned auto_increment not null primary key comment '主键',
    `message_id` varchar(128) not null comment '消息ID, 客户端生成, 可能不唯一',
    `device_number` varchar(128) not null comment '设备编号',
    `payload` varchar(1024) not null default '' comment '消息内容',
    `is_acked` tinyint unsigned not null default 0 comment '是否确认:0-未确认;1-已确认',
    `is_deleted` tinyint unsigned not null default 0 comment '是否删除:0-未删除;1-删除',
    `deleted_at` timestamp null comment '删除时间',
    `created_at` timestamp not null default current_timestamp comment '创建时间',
    `updated_at` timestamp not null default current_timestamp on update current_timestamp comment '更新时间'
) comment '紫外线灯 MQTT 消息表';

create table if not exists `uv_lamp_mqtt_received_messages`(
    `id` bigint unsigned auto_increment not null primary key comment '主键',
    `topic` varchar(128) not null comment '主题',
    `device_number` varchar(128) not null default '' comment '设备ID',
    `payload` varchar(1024) not null default '' comment '消息内容',
    `deleted_at` timestamp null comment '删除时间',
    `created_at` timestamp not null default current_timestamp comment '创建时间',
    `updated_at` timestamp not null default current_timestamp on update current_timestamp comment '更新时间'
) comment '紫外线灯 MQTT 接收消息表';

create table if not exists `uv_lamp_mqtt_notify_jobs`(
    `id` bigint unsigned auto_increment not null primary key comment '主键',
    `device_number` varchar(128) not null comment '设备编号',
    `notify_contents` varchar(1024) not null comment '通知内容',
    `is_completed` tinyint unsigned not null default 0 comment '是否完成:0-否;1-是',
    `retry_count` tinyint unsigned not null default 0 comment '重试次数',
    `next_retry_time` int unsigned not null default 0 comment '下次重试时间:时间戳(秒)',
    `deleted_at` timestamp null comment '删除时间',
    `created_at` timestamp not null default current_timestamp comment '创建时间',
    `updated_at` timestamp not null default current_timestamp on update current_timestamp comment '更新时间'
) comment '紫外线等 MQTT 通知任务表';