SELECT
    SUBSTR(InnerId, 1, 1) AS DomainAbbreviation,
    Date
FROM
    IIBb_Events
WHERE
    OuterId = '/*_OUTER_ID_*/'
ORDER BY
    Date ASC;