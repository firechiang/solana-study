import React from 'react';
import styled from '@emotion/styled';

import {REACT_GRAY} from '../../constants';

const NoProvider = () => {
    return (
        <StyledMain>
            <h2>找不到Phantom钱包</h2>
        </StyledMain>
    );
}

export default NoProvider;


const StyledMain = styled.main`
    padding: 20px;
    height: 100vh;
    background-color: ${REACT_GRAY};
`;