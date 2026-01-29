CREATE DATABASE IF NOT EXISTS uav
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_general_ci;

USE uav;

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    user_id VARCHAR(32)  NOT NULL COMMENT '用户ID',
    name VARCHAR(50) NOT NULL COMMENT '用户名',
    password VARCHAR(255) NOT NULL COMMENT '密码哈希值',
    role ENUM('superadmin','admin','user') NOT NULL COMMENT '用户角色',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ON UPDATE CURRENT_TIMESTAMP COMMENT '最近更新时间',
    PRIMARY KEY (user_id)
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_general_ci
COMMENT='用户表';


-- 无人机表
CREATE TABLE IF NOT EXISTS drones (
    drone_id VARCHAR(32) NOT NULL COMMENT '无人机ID，主键',
    name VARCHAR(50) NOT NULL COMMENT '无人机名称',
    model VARCHAR(50) NOT NULL COMMENT '无人机型号',
    status ENUM('idle','working','error','maintenance') NOT NULL DEFAULT 'idle' COMMENT '无人机状态：空闲/任务中/异常/维护',
    -- last_known_lat DECIMAL(9,6) DEFAULT NULL COMMENT '最后纬度，精度约0.1米',
    -- last_known_lng DECIMAL(9,6) DEFAULT NULL COMMENT '最后经度，精度约0.1米',
    battery TINYINT UNSIGNED NOT NULL COMMENT '电量百分比，0-100',
    activate BOOLEAN NOT NULL DEFAULT TRUE COMMENT '是否激活',
    -- created_at DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    PRIMARY KEY (drone_id)
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_general_ci
COMMENT='无人机表';


CREATE TABLE IF NOT EXISTS missions (
    mission_id VARCHAR(32) NOT NULL COMMENT '任务ID，主键',
    user_id VARCHAR(32) NOT NULL COMMENT '任务发起用户ID',
    drone_id VARCHAR(32) NOT NULL COMMENT '执行任务的无人机ID',
    target_lat DECIMAL(9,6) COMMENT '目标纬度',
    target_lng DECIMAL(9,6) COMMENT '目标经度',
    status ENUM('idle', 'working', 'returning', 'completed', 'error')
        NOT NULL DEFAULT 'idle' COMMENT '任务状态',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    started_at DATETIME COMMENT '任务开始时间',
    completed_at DATETIME COMMENT '任务完成时间',

    PRIMARY KEY (mission_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (drone_id) REFERENCES drones(drone_id)
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_general_ci
COMMENT='任务表';


CREATE TABLE IF NOT EXISTS events (
    event_id VARCHAR(32) NOT NULL COMMENT '事件ID，主键',
    mission_id VARCHAR(32) NOT NULL COMMENT '任务ID，外键',
    event_type ENUM('takeoff','landing','battery_low','obstacle_detected','return','error') NOT NULL COMMENT '事件类型',
    message TEXT COMMENT '事件描述',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',

    PRIMARY KEY (event_id),
    FOREIGN KEY (mission_id) REFERENCES missions(mission_id)
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_general_ci
COMMENT="事件表";

-- 任务日志表
CREATE TABLE IF NOT EXISTS logs (
    log_id INT AUTO_INCREMENT COMMENT '日志ID',
    -- user_id VARCHAR(32) COMMENT '操作用户ID',
    log_type ENUM('INFO','WARN','ERROR') NOT NULL COMMENT '日志类型',
    message TEXT NOT NULL COMMENT '日志内容',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '发生时间',

    PRIMARY KEY (log_id)
    -- INDEX idx_user_id (user_id),
    -- CONSTRAINT fk_logs_user
    --     FOREIGN KEY (user_id)
    --     REFERENCES users(user_id)
    --     ON DELETE SET NULL
    --     ON UPDATE CASCADE
)
ENGINE=InnoDB
DEFAULT CHARSET=utf8mb4
COLLATE=utf8mb4_general_ci
COMMENT='系统日志表';