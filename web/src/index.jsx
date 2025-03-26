/* @refresh reload */
import { render } from 'solid-js/web';
import { Router, Route } from "@solidjs/router";

import Home from "./Home"
import Login from "./Login"
import './index.css';

render(
    () => (
        <Router>
            <Route path="/" component={Home} />
            <Route path="/login" component={Login} />
        </Router>
    ),
    document.getElementById('root')
);
