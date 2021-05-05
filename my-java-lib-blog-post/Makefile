.PHONY: build

build:
	./gradlew build -x test

test: build
	./gradlew test

view-jar: build
	jar tf lib/build/libs/lib.jar
