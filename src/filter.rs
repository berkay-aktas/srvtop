use crate::scanner::DevProcess;

const DEV_PROCESS_NAMES: &[&str] = &[
    "node", "nodejs", "deno", "bun",
    "python", "python3", "uvicorn", "gunicorn",
    "java", "kotlin", "gradle", "mvn",
    "go", "air",
    "ruby", "rails", "puma", "unicorn",
    "php", "php-fpm",
    "cargo", "rustc",
    "dotnet",
    "postgres", "postgresql", "pg_isready",
    "redis-server", "redis",
    "mongod", "mongos",
    "mysql", "mysqld", "mariadb",
    "nginx", "caddy", "httpd", "apache2",
    "vite", "webpack", "esbuild", "turbopack", "next-server",
    "docker-proxy",
];

const DEV_PORTS: &[u16] = &[
    3000, 3001, 3002, 3003,
    4200, 4321,
    5000, 5001, 5173, 5174,
    5432, 5433,
    6379,
    8000, 8001, 8080, 8081, 8443, 8888,
    9000, 9090, 9229,
    27017,
];

pub fn is_dev_relevant(process: &DevProcess) -> bool {
    let name_lower = process.name.to_lowercase();

    for dev_name in DEV_PROCESS_NAMES {
        if name_lower == *dev_name || name_lower.starts_with(&format!("{}.", dev_name)) {
            return true;
        }
    }

    DEV_PORTS.contains(&process.port)
}

pub fn filter_dev(processes: Vec<DevProcess>) -> Vec<DevProcess> {
    processes.into_iter().filter(is_dev_relevant).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_dev_process_name() {
        assert!(is_dev_relevant(&DevProcess::test("node", 12345)));
        assert!(is_dev_relevant(&DevProcess::test("python3", 12345)));
        assert!(is_dev_relevant(&DevProcess::test("redis-server", 12345)));
    }

    #[test]
    fn matches_dev_process_name_with_extension() {
        assert!(is_dev_relevant(&DevProcess::test("node.exe", 12345)));
        assert!(is_dev_relevant(&DevProcess::test("mongod.exe", 12345)));
    }

    #[test]
    fn matches_dev_port() {
        assert!(is_dev_relevant(&DevProcess::test("unknown", 3000)));
        assert!(is_dev_relevant(&DevProcess::test("unknown", 5432)));
        assert!(is_dev_relevant(&DevProcess::test("unknown", 8080)));
        assert!(is_dev_relevant(&DevProcess::test("unknown", 27017)));
    }

    #[test]
    fn rejects_non_dev_process() {
        assert!(!is_dev_relevant(&DevProcess::test("svchost", 49152)));
        assert!(!is_dev_relevant(&DevProcess::test("explorer", 50000)));
    }

    #[test]
    fn filter_dev_removes_non_dev() {
        let processes = vec![
            DevProcess::test("node", 3000),
            DevProcess::test("svchost", 49152),
            DevProcess::test("unknown", 8080),
        ];
        let filtered = filter_dev(processes);
        assert_eq!(filtered.len(), 2);
    }
}
