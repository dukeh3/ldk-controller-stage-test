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
                    ssh jenkins@lp-dev "mkdir -p /opt/ldk-controller-stage-test"
                    scp Dockerfile docker-compose.yml jenkins@lp-dev:/opt/ldk-controller-stage-test/
                    ssh jenkins@lp-dev "cd /opt/ldk-controller-stage-test && docker compose build --no-cache && docker compose up -d"
                '''
            }
        }
    }

    post {
        success {
            sh 'ssh jenkins@lp-dev "docker ps --filter name=ldk-alice --filter name=ldk-bob"'
        }
    }
}
