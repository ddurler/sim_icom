# SIM_ICOM

Simulateur application ICOM

Cet outil simule le fonctionnement de la carte ICOM pour l'AFSEC+.

Usage:
    sim_icom com        Où com est le port série en communication avec l'AFSEC+
                        (Le port série 'fake' inhibe cette communication)

Le répertoire courant doit contenir un fichier 'database.csv' qui contient les informations
de la database de l'ICOM (fichier dont le contenu est identique au fichier database*.csv dans
la µSD de l'ICOM).

L'outil est également un serveur MODBUS/TCP pour interagir avec le contenu de la database.
