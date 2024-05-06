import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as crypto from 'node:crypto';

export class LearningStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const vpc = new ec2.Vpc(this, 'Vpc', {
      ipAddresses: ec2.IpAddresses.cidr("10.0.0.0/16"),
      maxAzs: 1,
      subnetConfiguration: [
        {
          cidrMask: 24,
          name: 'Public',
          subnetType: ec2.SubnetType.PUBLIC,
          mapPublicIpOnLaunch: false
        }
      ]
    });

    const bastionSg = new ec2.SecurityGroup(this, "from-bastion", {
      securityGroupName: "from-bastion",
      vpc: vpc
    });

    const allowFromBastionSg = new ec2.SecurityGroup(this, "allow-from-bastion", {
      securityGroupName: "allow-from-bastion",
      vpc: vpc
    });

    allowFromBastionSg.addIngressRule(
      bastionSg,
      ec2.Port.tcp(22)
    );

    const hash = crypto.createHash('md5');
    hash.update("learning-bucket");
    const bucket = new s3.Bucket(this, "bucket", {
      bucketName: `learning-bucket-${hash.digest('hex')}`,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true
    });

    const bastion = new ec2.BastionHostLinux(this, "bastion", {
      vpc: vpc,
      instanceName: "bastion",
      instanceType: ec2.InstanceType.of(
        ec2.InstanceClass.C7I,
        ec2.InstanceSize.XLARGE2
      ),
      securityGroup: bastionSg,
      blockDevices: [
        {
          deviceName: "/dev/xvda",
          volume: ec2.BlockDeviceVolume.ebs(30)
        }
      ]
    });

    bastion.role.addManagedPolicy(cdk.aws_iam.ManagedPolicy.fromAwsManagedPolicyName("AdministratorAccess"));
  }
}
