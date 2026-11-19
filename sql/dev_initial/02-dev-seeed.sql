-- Insert root organization at id=0
INSERT INTO "organization" (id, name) VALUES (0, 'root');

-- Insert root user at id=0
INSERT INTO "user" (id, org_id, username) VALUES (0, 0, 'root');

-- Insert demo user and organization
INSERT INTO "organization"
    (name) VALUES
    ('org1');

INSERT INTO "user" (org_id, username) VALUES (1000, 'demo1');

INSERT INTO "country" (code) VALUES ('FRA');

INSERT INTO "ancillary_market" (country, service_type) VALUES
    ('FRA', 'Fcr'), ('FRA', 'Afrr'), ('FRA', 'AfrrFree');

INSERT INTO "bsp"
    (name,country,org_id,service_type) VALUES
    ('demo_bsp','FRA',1000,'Afrr');

INSERT INTO "brp"
    (name,country,org_id) VALUES
    ('demo_brp','FRA',1000);

INSERT INTO "asset"
    (name,country,brp_id,org_id) VALUES
    ('demo_asset','FRA',1000,1000);

INSERT INTO "asset_bsp_association"
    (asset_id, bsp_id, org_id, country, service_type)
    VALUES (1000, 1000, 1000, 'FRA', 'Afrr');
