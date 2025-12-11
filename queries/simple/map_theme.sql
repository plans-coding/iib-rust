SELECT
    * FROM IIBb_Events
WHERE
    AdditionalNotes LIKE "%| ${parameter} }%";