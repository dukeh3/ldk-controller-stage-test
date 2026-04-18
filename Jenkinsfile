pipeline {
    agent { label 'docker-rust-agent-1' }

    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        disableConcurrentBuilds()
    }

    stages {
        stage('Deploy to lp-dev') {
            steps {
                sh '''
                    set -eux
                    ssh root@172.16.10.100 "mkdir -p /opt/ldk-controller-stage-test"
                    scp Dockerfile docker-compose.yml root@172.16.10.100:/opt/ldk-controller-stage-test/
                    ssh root@172.16.10.100 "chown -R 100:101 /opt/ldk-alice /opt/ldk-bob"
                    ssh root@172.16.10.100 "cd /opt/ldk-controller-stage-test && docker compose down && docker compose build --no-cache && docker compose up -d"
                '''
            }
        }

        stage('Smoke Test') {
            steps {
                sh '''
                    set -eux
                    cd nwc-check && cargo build --release
                '''
                // Wait for containers to be fully ready
                sleep(time: 10, unit: 'SECONDS')
                sh '''
                    set -eux
                    scp nwc-check/target/release/nwc-check root@172.16.10.100:/tmp/
                    ssh root@172.16.10.100 "/tmp/nwc-check ws://172.16.10.101:7777 alice=f1a7ecc5f92ef37ef412cfe6c2218e13ae374a27350b612ea718b2ef55506251 bob=832fd6ab8c12dae36a67d951dfe7c22b4edd96b127aaa924844339ddac1e3eff"
                '''
            }
        }
    }

    post {
        success {
            sh 'ssh root@172.16.10.100 "docker ps --filter name=ldk-alice --filter name=ldk-bob"'
        }
    }
}
