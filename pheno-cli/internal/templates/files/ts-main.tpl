import express, { Request, Response } from 'express';
import { EntityService } from './application/service';
import { InMemoryEntityRepository } from './adapters/repository';

const repository = new InMemoryEntityRepository();
const service = new EntityService(repository);

const app = express();
app.use(express.json());

app.get('/health', (_req: Request, res: Response) => {
  res.json({ status: 'healthy' });
});

app.post('/entities', async (req: Request, res: Response) => {
  try {
    const { name, description } = req.body;
    const entity = await service.create({ name, description });
    res.status(201).json(entity.toJSON());
  } catch (error) {
    res.status(400).json({ error: (error as Error).message });
  }
});

app.get('/entities/:id', async (req: Request, res: Response) => {
  try {
    const entity = await service.getById(req.params.id);
    res.json(entity.toJSON());
  } catch (error) {
    res.status(404).json({ error: (error as Error).message });
  }
});

app.put('/entities/:id', async (req: Request, res: Response) => {
  try {
    const { name, description } = req.body;
    const entity = await service.update(req.params.id, { name, description });
    res.json(entity.toJSON());
  } catch (error) {
    res.status(400).json({ error: (error as Error).message });
  }
});

app.delete('/entities/:id', async (req: Request, res: Response) => {
  try {
    await service.delete(req.params.id);
    res.status(204).send();
  } catch (error) {
    res.status(400).json({ error: (error as Error).message });
  }
});

app.get('/entities', async (_req: Request, res: Response) => {
  try {
    const entities = await service.list();
    res.json(entities.map(e => e.toJSON()));
  } catch (error) {
    res.status(500).json({ error: (error as Error).message });
  }
});

const PORT = process.env.PORT || 8080;
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
