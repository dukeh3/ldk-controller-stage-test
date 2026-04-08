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
                    ssh root@172.16.10.100 "cd /opt/ldk-controller-stage-test && docker compose build --no-cache && docker compose up -d"
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
