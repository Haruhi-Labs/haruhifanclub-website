ALTER TABLE branch_brand ADD COLUMN logo_path TEXT;

UPDATE branch_brand
SET logo_path = COALESCE(
    NULLIF(TRIM(logo_light_path), ''),
    NULLIF(TRIM(logo_mark_path), ''),
    NULLIF(TRIM(logo_dark_path), '')
)
WHERE logo_path IS NULL OR TRIM(logo_path) = '';
