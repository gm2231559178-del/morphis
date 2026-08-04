-- One notify payload shape: a material document must be (re)indexed.
-- Emitted by the materials table and by every child table whose rows roll up
-- into a material document (sizes, colorways, material_features,
-- feature_attributes). Consumers of materials_channel re-index the parent.
CREATE OR REPLACE FUNCTION notify_material(mat_no_val TEXT)
RETURNS VOID AS $$
BEGIN
  PERFORM pg_notify(
    'materials_channel',
    json_build_object(
      'meta', json_build_object('event_type', 'material'),
      'data', json_build_object('mat_no', mat_no_val)
    )::text
  );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION notify_material_change()
RETURNS trigger AS $$
DECLARE
  mat_no_val TEXT;
BEGIN
  IF TG_OP = 'DELETE' THEN
    mat_no_val := OLD.mat_no;
  ELSE
    mat_no_val := NEW.mat_no;
  END IF;

  PERFORM notify_material(mat_no_val);

  IF TG_OP = 'DELETE' THEN
    RETURN OLD;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS material_change_trigger ON materials;
CREATE TRIGGER material_change_trigger
AFTER INSERT OR UPDATE OR DELETE ON materials
FOR EACH ROW EXECUTE FUNCTION notify_material_change();

-- Child tables: resolve the parent mat_no and emit the same material event so
-- the producer re-indexes the parent document (feature_attributes joins up
-- through material_features).
CREATE OR REPLACE FUNCTION notify_material_child_change()
RETURNS trigger AS $$
DECLARE
  mat_no_val TEXT;
BEGIN
  IF TG_TABLE_NAME = 'feature_attributes' THEN
    SELECT mat_no INTO mat_no_val
      FROM material_features
      WHERE id = (CASE WHEN TG_OP = 'DELETE' THEN OLD.feature_id ELSE NEW.feature_id END);
  ELSE
    IF TG_OP = 'DELETE' THEN
      mat_no_val := OLD.mat_no;
    ELSE
      mat_no_val := NEW.mat_no;
    END IF;
  END IF;

  IF mat_no_val IS NOT NULL THEN
    PERFORM notify_material(mat_no_val);
  END IF;

  IF TG_OP = 'DELETE' THEN
    RETURN OLD;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS sizes_change_trigger ON sizes;
CREATE TRIGGER sizes_change_trigger
AFTER INSERT OR UPDATE OR DELETE ON sizes
FOR EACH ROW EXECUTE FUNCTION notify_material_child_change();

DROP TRIGGER IF EXISTS colorways_change_trigger ON colorways;
CREATE TRIGGER colorways_change_trigger
AFTER INSERT OR UPDATE OR DELETE ON colorways
FOR EACH ROW EXECUTE FUNCTION notify_material_child_change();

DROP TRIGGER IF EXISTS material_features_change_trigger ON material_features;
CREATE TRIGGER material_features_change_trigger
AFTER INSERT OR UPDATE OR DELETE ON material_features
FOR EACH ROW EXECUTE FUNCTION notify_material_child_change();

DROP TRIGGER IF EXISTS feature_attributes_change_trigger ON feature_attributes;
CREATE TRIGGER feature_attributes_change_trigger
AFTER INSERT OR UPDATE OR DELETE ON feature_attributes
FOR EACH ROW EXECUTE FUNCTION notify_material_child_change();
