import os


def define_env(env):
    """
    This is the hook for the variables, macros and filters.
    """

    @env.macro
    def encryption_key_generator():
        "Show encryption key generator"
        with open(os.path.join("components", "generate_encryption_key.html"), 'r') as file:
            return file.read()