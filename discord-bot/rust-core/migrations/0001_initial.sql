-- Guild configuration
CREATE TABLE IF NOT EXISTS guild_config (
    guild_id            BIGINT PRIMARY KEY,
    auto_role_id        BIGINT,
    mod_role_id         BIGINT,
    log_channel_id      BIGINT,
    welcome_channel_id  BIGINT,
    verification_channel_id BIGINT,
    roles_channel_id    BIGINT,
    verification_enabled BOOLEAN DEFAULT true,
    spam_protection     BOOLEAN DEFAULT true,
    raid_protection     BOOLEAN DEFAULT true,
    link_filter         BOOLEAN DEFAULT true,
    created_at          TIMESTAMPTZ DEFAULT NOW(),
    updated_at          TIMESTAMPTZ DEFAULT NOW()
);

-- Warning history
CREATE TABLE IF NOT EXISTS warnings (
    id              SERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL,
    guild_id        BIGINT NOT NULL,
    reason          TEXT NOT NULL,
    moderator_id    BIGINT,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_warnings_user_guild
    ON warnings (user_id, guild_id);

-- Moderation logs
CREATE TABLE IF NOT EXISTS mod_logs (
    id          SERIAL PRIMARY KEY,
    guild_id    BIGINT NOT NULL,
    user_id     BIGINT NOT NULL,
    action      VARCHAR(50) NOT NULL,  -- ban, timeout, warn, kick
    reason      TEXT,
    moderator_id BIGINT,
    duration    INTEGER,               -- in minutes (for timeout)
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mod_logs_guild
    ON mod_logs (guild_id, created_at DESC);

-- Raid events
CREATE TABLE IF NOT EXISTS raid_events (
    id              SERIAL PRIMARY KEY,
    guild_id        BIGINT NOT NULL,
    affected_users  INTEGER DEFAULT 0,
    action_taken    VARCHAR(100),
    started_at      TIMESTAMPTZ DEFAULT NOW(),
    ended_at        TIMESTAMPTZ
);
