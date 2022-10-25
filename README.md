# Running with Helm

	1. Define variables
	```bash
	AWS_ACCESS_KEY_ID=
	AWS_SECRET_ACCESS_KEY=
	AWS_S3_BUCKET=
	AWS_REGION=
	```
	1. run helm
	```bash
  helm install myapp app \
		--set env.AWS_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID \
		--set env.AWS_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY \
		--set env.AWS_S3_BUCKET=$AWS_S3_BUCKET \
		--set env.AWS_REGION=$AWS_REGION \
		--values app/values.yaml
	```
