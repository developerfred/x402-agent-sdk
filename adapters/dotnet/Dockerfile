# C# .NET Test Dockerfile
FROM mcr.microsoft.com/dotnet/sdk:8.0

WORKDIR /app

# Copy project files
COPY adapters/dotnet/*.csproj /app/
COPY adapters/dotnet/*.cs /app/

# Restore and test
RUN dotnet restore
RUN dotnet test --verbosity normal

CMD ["dotnet", "test", "--verbosity", "normal"]
