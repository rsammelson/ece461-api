use super::*;

#[test]
fn parse_package_json() {
    let data = r#"{
  "name": "elm-test",
  "version": "0.19.1-revision12",
  "description": "Run elm-test suites.",
  "main": "elm-test.js",
  "engines": {
    "node": ">=12.20.0"
  },
  "scripts": {
    "prepare": "elm-tooling install",
    "test": "npm run check && npm run test-only",
    "flow": "flow",
    "lint": "eslint --report-unused-disable-directives .",
    "review": "cd elm && elm-review",
    "elm-test": "cd elm && node ../bin/elm-test",
    "test-only": "mocha tests && npm run elm-test",
    "check": "flow check && npm run lint && npm run format:check && npm run review",
    "format:check": "prettier --check . && elm-format elm --validate",
    "format:write": "prettier --write . && elm-format elm --yes"
  },
  "repository": {
    "type": "git",
    "url": "git+https://github.com/rtfeldman/node-test-runner.git"
  },
  "bin": {
    "elm-test": "bin/elm-test"
  },
  "files": [
    "bin",
    "lib",
    "templates",
    "elm/src"
  ],
  "keywords": [
    "elm",
    "elm-test",
    "cli"
  ],
  "author": "Richard Feldman",
  "license": "BSD-3-Clause",
  "bugs": {
    "url": "https://github.com/rtfeldman/node-test-runner/issues"
  },
  "homepage": "https://github.com/rtfeldman/node-test-runner#readme",
  "dependencies": {
    "chalk": "^4.1.2",
    "chokidar": "^3.5.3",
    "commander": "^9.4.1",
    "cross-spawn": "^7.0.3",
    "elm-solve-deps-wasm": "^1.0.2",
    "glob": "^8.0.3",
    "graceful-fs": "^4.2.10",
    "split": "^1.0.1",
    "which": "^2.0.2",
    "xmlbuilder": "^15.1.1"
  },
  "devDependencies": {
    "elm-review": "2.9.1",
    "elm-tooling": "1.10.0",
    "eslint": "8.31.0",
    "eslint-plugin-mocha": "10.1.0",
    "flow-bin": "0.180.0",
    "mocha": "9.2.2",
    "prettier": "2.8.1",
    "strip-ansi": "6.0.0",
    "xml2js": "0.5.0"
  }
}
"#;
    let package_json: PackageJson = serde_json::from_str(data).unwrap();

    let PackageJson::Deep {
        name,
        version,
        repository,
        dependencies,
    } = package_json
    else {
        panic!("Expected package_json to be the Deep variant\n{:?}", package_json);
    };

    let Some(dependencies) = dependencies
    else {
        panic!("Expected dependencies to be some");
    };

    assert_eq!(name, "elm-test");
    assert_eq!(version, "0.19.1-revision12".parse().unwrap());
    assert_eq!(
        repository,
        Repository {
            url: "git+https://github.com/rtfeldman/node-test-runner.git".to_string()
        }
    );

    assert_eq!(
        dependencies,
        HashMap::from([
            ("chalk".to_string(), "^4.1.2".to_string()),
            ("chokidar".to_string(), "^3.5.3".to_string()),
            ("commander".to_string(), "^9.4.1".to_string()),
            ("cross-spawn".to_string(), "^7.0.3".to_string()),
            ("elm-solve-deps-wasm".to_string(), "^1.0.2".to_string()),
            ("glob".to_string(), "^8.0.3".to_string()),
            ("graceful-fs".to_string(), "^4.2.10".to_string()),
            ("split".to_string(), "^1.0.1".to_string()),
            ("which".to_string(), "^2.0.2".to_string()),
            ("xmlbuilder".to_string(), "^15.1.1".to_string()),
        ])
    );

    assert_eq!(version::score_versionreq_pinned(dependencies), 0.);
}

#[test]
fn parse_package_json_simple() {
    let data = r#"{
  "name": "fake-package",
  "version": "1.2.3",
  "repository": "https://github.com/fake/repo"
}
"#;
    let package_json: PackageJson = serde_json::from_str(data).unwrap();

    let PackageJson::Flat {
        name,
        version,
        repository,
        dependencies,
    } = package_json
    else {
        panic!("Expected package_json to be the Flat variant\n{:?}", package_json);
    };

    assert_eq!(name, "fake-package");
    assert_eq!(version, "1.2.3".parse().unwrap());
    assert_eq!(repository, "https://github.com/fake/repo");
    assert_eq!(dependencies, None);
}
