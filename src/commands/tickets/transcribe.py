import chat_exporter # pip3 install git+https://github.com/MaLoefUDS/DiscordChatExporterPy.git
import sys
import discord

intents = discord.Intents.default()
intents.guilds = True
intents.guild_messages = True
intents.message_content = True

client = discord.Client(intents=intents)

@client.event
async def on_ready():
    await chat_exporter.quick_export(channel=sys.argv[2],
                                     guild=sys.argv[3],
                                     ticket_id=sys.argv[4],
                                     client=client)
    await client.close()

client.run(sys.argv[1])
