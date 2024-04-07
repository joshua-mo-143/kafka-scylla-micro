##
# Project Title
#
# @file
# @version 0.1
up:
	docker compose down && docker compose up --build -d

logs:
	docker compose logs | grep "buyer"
# end
