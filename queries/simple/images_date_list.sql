SELECT
    bewa_Overview.OuterId,
    SUBSTR(bewb_Events.InnerId, 1, 1) AS DomainAbbreviation,
    bewb_Events.Date
FROM
    bewa_Overview
LEFT JOIN bewb_Events ON bewb_Events.InnerId = bewa_Overview.InnerId
WHERE
    OuterId = '/*_OUTER_ID_*/'
ORDER BY
    Date ASC;