INSERT INTO Authors(author_id, name, date_of_birth, date_of_death, description) VALUES
    (1, 'Mihai Eminescu', unixepoch('1850-01-15'), unixepoch('1889-06-15'), 'Cel mai pare poet al României'),
    (2, 'Howard Phillips Lovecraft', unixepoch('1890-08-20'), unixepoch('1937-03-15'), 'Scriitor american, cunoscut pentru literatura sa de groază, mai ales pentru Miturile lui Chtulhu'),
    (3, 'Eiji Mikage', unixepoch('1983-07-27'), NULL, 'Scriitor japonez, cunoscut pentru The Empty Box and Zeroth Maria'),
    (4, 'Ion Creangă', unixepoch('1837-03-01'), unixepoch('1889-12-31'), 'Povestitor român, cunoscut pentru Amintiri din Copilărie și Harap-Alb'),
    (5, 'Ion Luca Caragiale', unixepoch('1852-02-01'), unixepoch('1912-06-09'), 'Dramaturg, nuvelist, pamfletar, poet, scriitor, director de teatru, comentator politic și ziarist român.');

INSERT INTO Books(title, author_id, publish_date, publisher, count, synopsis, language) VALUES
    ('The Empty Box and Zeroth Maria, vol. 1', 3, unixepoch('2009-01-07'), 'Yen Press', 3, 'Kazuki Hoshino meets Aya Otonashi, whose name is later revealed to be Maria', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 2', 3, unixepoch('2010-11-17'), 'Yen Press', 3, 'Kazuki Hoshino''s body is possesed by a mysterious force', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 3', 3, unixepoch('2011-02-03'), 'Yen Press', 0, 'Kazuki Hoshino and his friends are dragged into a death game', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 4', 3, unixepoch('2012-01-01'), 'Yen Press', 0, 'Kazuki Hoshino and his friends witness the conclusion of the death game', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 5', 3, unixepoch('2013-08-03'), 'Yen Press', 0, 'One of Kazuki Hoshino''s friends gets powers that allow him to control the minds of people', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 6', 3, unixepoch('2014-11-11'), 'Yen Press', 2, 'Kazuki Hoshino stops his friend''s evil plans of destroying the world', 'en'),
    ('The Empty Box and Zeroth Maria, vol. 7', 3, unixepoch('2015-06-10'), 'Yen Press', 2, 'Kazuki Hoshino and Maria Otonashi marry', 'en'),
    ('Colecție de poezii', 1, unixepoch('2023-01-23'), 'Editura Ciuperca', 15, 'O colectie de poezii scrise de Mihai Eminescu', 'ro'),
    ('Dl. Goe', 5, unixepoch('2018-01-01'), 'Galaxia Copiilor', 10, 'O pretioasa lectie de viata ambalata in hohote de ras.', 'ro'),
    ('Nuvele si teatru', 5, unixepoch('2022-01-01'), 'Rolcris', 10, 'O colectie de scrieri a lui I.L. Caragiale', 'ro'),
    ('Amintiri din copilărie', 4, unixepoch('2012-02-23'), 'Gramar', 10, 'Cartea descrie intr-un mod spumos copilaria lui Ion Creanga', 'ro'),
    ('The Complete Tales of H.P. Lovecraft', 2, unixepoch('2019-10-10'), 'Rock Point', 3, 'The Complete Tales of H.P. Lovecraft collects the author''s novel, four novellas, and fifty-three short stories.', 'en'),
    ('Chemarea lui Cthulhu si alte povestiri', 2, unixepoch('2019-11-02'), 'Polirom', 6, 'O colecție de 12 povestiri ce conține nucleul universului mitic al lui H.P. Lovecraft', 'ro');
