SELECT json_group_object(AttributeGroup, json(group_data))
FROM (
    SELECT
        COALESCE(AttributeGroup, 'General') AS AttributeGroup,
        json_group_object(COALESCE(Attribute, 'Unknown'), Value) AS group_data
    FROM bewx_Settings
    WHERE AttributeGroup NOT IN ('Definition', 'Extension')
    GROUP BY AttributeGroup
);
