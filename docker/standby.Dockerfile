
FROM postgres:18

COPY ./init-standby.sh ./init.sh


RUN chmod +x ./init.sh
	
ENTRYPOINT ./init.sh




