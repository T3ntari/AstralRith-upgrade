-- Personalization, client runtime and bot storage
CREATE TABLE IF NOT EXISTS skins (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    file_name TEXT NOT NULL,
    model_type TEXT NOT NULL DEFAULT 'classic', -- classic | slim
    source TEXT NOT NULL DEFAULT 'local',       -- local | imported | edited | account
    account_id TEXT NULL,                       -- minecraft account uuid if bound
    favorite INTEGER NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    last_used INTEGER NULL,
    created INTEGER NOT NULL,
    modified INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS client_profiles (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    modules TEXT NOT NULL DEFAULT '{}',   -- JSON: module id -> config
    keybinds TEXT NOT NULL DEFAULT '{}',
    hud TEXT NOT NULL DEFAULT '{}',
    render TEXT NOT NULL DEFAULT '{}',
    version_compatibility TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT FALSE,
    created INTEGER NOT NULL,
    modified INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS bot_profiles (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 25565,
    version TEXT NULL,
    auth_type TEXT NOT NULL DEFAULT 'offline', -- offline | microsoft
    username TEXT NOT NULL,
    script TEXT NULL,
    plugins TEXT NOT NULL DEFAULT '[]',
    created INTEGER NOT NULL,
    modified INTEGER NOT NULL
);
