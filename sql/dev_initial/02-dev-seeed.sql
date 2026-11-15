INSERT INTO "organization"
    (name) VALUES
    ('org1');

INSERT INTO "bsp"
    (name,country,org_id) VALUES
    ('demo_bsp','FRA',1000);

INSERT INTO "brp"
    (name,country,org_id) VALUES
    ('demo_brp','FRA',1000);

INSERT INTO "asset"
    (name,country,bsp_id,brp_id,org_id) VALUES
    ('demo_asset','FRA',1000,1000,1000);
