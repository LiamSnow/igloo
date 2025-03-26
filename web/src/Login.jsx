import styles from './Login.module.scss';

export default function Login() {
    return (
        <div class={styles.wrapper}>
            <div>
                <h1>Login</h1>
                <form method="POST" class={styles.form}>
                    <div>
                        <label for="username">Username</label>
                        <input
                            id="username"
                            name="username"
                            type="text"
                        />
                    </div>
                    <div>
                        <label for="password">Password</label>
                        <input
                            id="password"
                            name="password"
                            type="password"
                        />
                    </div>
                    <button type="submit">Login</button>
                </form>
            </div>
        </div>
    );
}
