-- Multi-tenant schema: tenants, workspaces, and workspace-scoped members
-- Migration: 0003_multi_tenant_schema.sql

-- Create tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    plan VARCHAR(50) NOT NULL DEFAULT 'free',
    max_members INTEGER DEFAULT 100,
    max_rooms INTEGER DEFAULT 50,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create workspaces table (belongs to a tenant)
CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, slug)
);

-- Update members table to reference workspaces
-- Note: This is additive to preserve existing structure
-- Add workspace_id column if not exists
ALTER TABLE members ADD COLUMN IF NOT EXISTS workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE members ADD COLUMN IF NOT EXISTS external_id VARCHAR(255);
ALTER TABLE members ADD COLUMN IF NOT EXISTS display_name VARCHAR(255);

-- Create unique constraint for workspace-scoped external_id
CREATE UNIQUE INDEX IF NOT EXISTS idx_members_workspace_external_id 
    ON members (workspace_id, external_id) 
    WHERE workspace_id IS NOT NULL AND external_id IS NOT NULL;

-- Indexes for tenant/workspace lookups
CREATE INDEX IF NOT EXISTS idx_workspaces_tenant_id ON workspaces (tenant_id);
CREATE INDEX IF NOT EXISTS idx_members_workspace_id ON members (workspace_id);

-- Trigger for updated_at on tenants
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants;
CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
