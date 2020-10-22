


function App() {
    const [devs, setDevs] = React.useState([
      {
        name: "Sabino",
        score: 5,
      },
      {
        name: "Emilio",
        score: 3,
      },
      {
        name: "Charie Brown",
        score: 2,
      },
    ]);
  
    const addInDev = (dev) => {
      const i = devs.indexOf(dev);
      const copyDevs = [...devs];
      copyDevs[i].score++;
      copyDevs[i].isChanging = true;
      setDevs(copyDevs.sort((a, b) => b.score - a.score));
    };
  
    return (
      <>
        <table border="1">
          {devs.map((dev) => (
            <tr
              key={dev.name}
              style={{
                zIndex: dev.isChanging ? 10 : 5,
                position: "absolute",
                top: devs.indexOf(dev) * 40,
                transition: "1s ease top",
              }}
            >
              <td>{dev.name}</td>
              <td>{dev.score}</td>
            </tr>
          ))}
        </table>
  
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
        <br />
  
        {devs.map((dev) => (
          <div>
            <button onClick={() => addInDev(dev)}>{dev.name}</button>
          </div>
        ))}
      </>
    );
    
    return <p>Hello World!</p>
  }
  
  ReactDOM.render(
    <App />,
    document.getElementById('root')
  );
  
  
  //export default App;
  