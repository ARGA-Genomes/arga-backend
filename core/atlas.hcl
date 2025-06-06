env "arga" {
    url = getenv("DATABASE_URL")
    dev = getenv("MIGRATOR_DATABASE_URL")
    src = "file://schema.sql"

    migrations {
        dir = "file://migrations"
        baseline = "20250605060808"
    }

    format {
        migrate {
            diff = "{{ sql . ' ' }}"
        }
    }
}
